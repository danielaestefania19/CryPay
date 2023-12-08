use candid::{candid_method, CandidType, Decode, Deserialize, Encode};
use ic_cdk::api::call::RejectionCode;
use ic_cdk::export::candid::Principal;
use ic_ledger_types::{
    AccountIdentifier, BlockIndex, Memo, Subaccount, Tokens, DEFAULT_FEE, DEFAULT_SUBACCOUNT,
    MAINNET_LEDGER_CANISTER_ID,
};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use serde_derive::Serialize;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE_PAYMENT: u32 = 200;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum PaymentStatus {
    NotExists,
    New,
    Paid,
    Completed,
    Refunded,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash, PartialEq)]
pub struct Petunia {
    ledger_canister_id: Principal,
    subaccount: Option<Subaccount>,
    transaction_fee: Tokens,
}

impl Default for Petunia {
    fn default() -> Self {
        Petunia {
            ledger_canister_id: MAINNET_LEDGER_CANISTER_ID,
            subaccount: None,
            transaction_fee: DEFAULT_FEE,
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Hash)]
pub struct TransferArgs {
    status: u8,
    memo: Memo,
    amount: Tokens,
    to_principal: Principal,
    to_subaccount: Option<Subaccount>,
}

impl Storable for TransferArgs {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for TransferArgs {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE_PAYMENT;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    static PETUNIA: RefCell<Petunia> = RefCell::new(Petunia::default());
    static PAYMENTS: RefCell<StableBTreeMap<u64, TransferArgs, Memory>> = RefCell::new(StableBTreeMap::init(
        MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
    ));
}

#[ic_cdk::init]
#[candid_method(init)]
fn init(petunia: Petunia) {
    PETUNIA.with(|p| p.replace(petunia));
}

#[ic_cdk::query]
fn get_address() -> Principal {
    ic_cdk::caller()
}

#[ic_cdk::query]
#[candid_method(query)]
fn get_pay_status(external_payment_id: u64) -> String {
    PAYMENTS.with(|p| {
        let payments = p.borrow();
        let payment = payments.get(&external_payment_id);
        match payment {
            Some(payment) => match payment.status {
                0 => "NotExists".to_string(),
                1 => "New".to_string(),
                2 => "Paid".to_string(),
                3 => "Completed".to_string(),
                4 => "Refunded".to_string(),
                _ => format!("Pago no Existe"),
            },
            None => format!("Pago no Existe"),
        }
    })
}

#[ic_cdk::query]
#[candid_method(query)]
fn check_if_payment_exists(external_payment_id: u64) -> bool {
    PAYMENTS.with(|p| {
        let payments = p.borrow();
        let payment = payments.get(&external_payment_id);
        match payment {
            Some(payment) => payment.status > 0,
            None => false,
        }
    })
}

#[ic_cdk::query]
#[candid_method(query)]
fn get_price(external_payment_id: u64) -> Option<Tokens> {
    PAYMENTS.with(|p| {
        let payments = p.borrow();
        let payment = payments.get(&external_payment_id);
        match payment {
            Some(payment) => {
                if payment.status > 0 {
                    ic_cdk::println!(
                        "Precio del pago {}: {:?}",
                        external_payment_id,
                        payment.amount
                    );
                    Some(payment.amount)
                } else {
                    ic_cdk::println!(
                        "Error: El pago {} no existe o no está pagado.",
                        external_payment_id
                    );
                    None
                }
            }
            None => {
                ic_cdk::println!("Error: El pago {} no existe.", external_payment_id);
                None
            }
        }
    })
}


#[ic_cdk::update]
#[candid_method(update)]
fn start_new_payment(
    external_payment_id: u64,
    memo: Memo,
    amount: Tokens,
    to_principal: Principal,
    to_subaccount: Option<Subaccount>,
) -> Result<String, String> {
    let payment_exists = check_if_payment_exists(external_payment_id);

    if !payment_exists {
        PAYMENTS.with(|p| {
            let mut payments = p.borrow_mut();
            let new_payment = TransferArgs {
                amount,
                memo,
                status: 1,
                to_principal,
                to_subaccount,
            };
            payments.insert(external_payment_id, new_payment);
        });

        Ok(format!("Pago {} iniciado correctamente.", external_payment_id))
    } else {
        Err(format!("Error: El pago {} ya existe.", external_payment_id))
    }
}
#[ic_cdk::update]
#[candid_method(update)]
async fn pay(external_payment_id: u64) -> Result<BlockIndex, String> {
    let payment_status = get_pay_status(external_payment_id);

    if payment_status == "New" {
        let payment = PAYMENTS.with(|p| {
            let payments = p.borrow();
            payments.get(&external_payment_id).clone()
        });

        if let Some(payment) = payment {
            //Clone the payment details
            let transfer_args = TransferArgs {
                status: 1, // Set status to "New"
                memo: payment.memo.clone(),
                amount: payment.amount,
                to_principal: payment.to_principal.clone(),
                to_subaccount: payment.to_subaccount.clone(),
            };

            ic_cdk::println!(
                "Transferring {} tokens to principal {} subaccount {:?}",
                &transfer_args.amount,
                &transfer_args.to_principal,
                &transfer_args.to_subaccount
            );

            let ledger_canister_id = PETUNIA.with(|p| p.borrow().ledger_canister_id);
            let to_subaccount = transfer_args.to_subaccount.unwrap_or(DEFAULT_SUBACCOUNT);

            let transfer_args = PETUNIA.with(|p| {
                let petunia = p.borrow();
                ic_ledger_types::TransferArgs {
                    memo: transfer_args.memo,
                    amount: transfer_args.amount,
                    fee: petunia.transaction_fee,
                    from_subaccount: petunia.subaccount,
                    to: AccountIdentifier::new(&transfer_args.to_principal, &to_subaccount),
                    created_at_time: None,
                }
            });

            

            ic_ledger_types::transfer(ledger_canister_id, transfer_args)
                .await
                .map_err(|e| format!("failed to call ledger: {:?}", e))?
                .map_err(|e| format!("ledger transfer error {:?}", e))
        } else {
            Err(format!(
                "Payment details not found for payment ID {}",
                external_payment_id
            ))
        }
    } else {
        Err(format!(
            "Error: Payment {} does not have the expected status 'New'",
            external_payment_id
        ))
    }
}