export const idlFactory = ({ IDL }) => {
  const Memo = IDL.Nat64;
  const Tokens = IDL.Record({ 'e8s' : IDL.Nat64 });
  const TransferArgs = IDL.Record({
    'status' : IDL.Nat8,
    'to_principal' : IDL.Principal,
    'to_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'memo' : Memo,
    'from_subaccount' : IDL.Opt(IDL.Vec(IDL.Nat8)),
    'amount' : Tokens,
  });
  const TransferResult = IDL.Variant({ 'Ok' : TransferArgs, 'Err' : IDL.Text });
  const start_pay_result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  return IDL.Service({
    'check_if_payment_exists' : IDL.Func([IDL.Nat64], [IDL.Bool], ['query']),
    'get_address' : IDL.Func([], [IDL.Principal], ['query']),
    'get_pay_status' : IDL.Func([IDL.Nat64], [IDL.Text], ['query']),
    'get_payment_details' : IDL.Func([IDL.Nat64], [TransferResult], ['query']),
    'get_price' : IDL.Func([IDL.Nat64], [IDL.Opt(Tokens)], ['query']),
    'pay' : IDL.Func([IDL.Nat64], [TransferResult], []),
    'start_new_payment' : IDL.Func(
        [IDL.Nat64, Memo, Tokens, IDL.Principal, IDL.Opt(IDL.Vec(IDL.Nat8))],
        [start_pay_result],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
