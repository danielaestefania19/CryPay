import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export type Memo = bigint;
export type PaymentDetails = { 'Ok' : TransferArgs } |
  { 'Err' : string };
export type PaymentStatus = { 'New' : null } |
  { 'Refunded' : null } |
  { 'Paid' : null } |
  { 'NotExists' : null } |
  { 'Completed' : null };
export interface Petunia {
  'transaction_fee' : Tokens,
  'subaccount' : [] | [Uint8Array | number[]],
  'ledger_canister_id' : Principal,
}
export type PetuniaError = { 'PaymentExists' : null } |
  { 'PaymentDoesNotExist' : null } |
  { 'InvalidPrice' : null } |
  { 'NotOwner' : null } |
  { 'InsufficientFunds' : null };
export interface Tokens { 'e8s' : bigint }
export interface TransferArgs {
  'status' : number,
  'to_principal' : Principal,
  'to_subaccount' : [] | [Uint8Array | number[]],
  'memo' : Memo,
  'from_subaccount' : [] | [Uint8Array | number[]],
  'amount' : Tokens,
}
export type TransferResult = { 'Ok' : TransferArgs } |
  { 'Err' : string };
export type start_pay_result = { 'Ok' : string } |
  { 'Err' : string };
export interface _SERVICE {
  'check_if_payment_exists' : ActorMethod<[bigint], boolean>,
  'get_address' : ActorMethod<[], Principal>,
  'get_pay_status' : ActorMethod<[bigint], string>,
  'get_payment_details' : ActorMethod<[bigint], TransferResult>,
  'get_price' : ActorMethod<[bigint], [] | [Tokens]>,
  'pay' : ActorMethod<[bigint], TransferResult>,
  'start_new_payment' : ActorMethod<
    [bigint, Memo, Tokens, Principal, [] | [Uint8Array | number[]]],
    start_pay_result
  >,
}
