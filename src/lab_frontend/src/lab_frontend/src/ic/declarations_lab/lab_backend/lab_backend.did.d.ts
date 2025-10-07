import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface ExperimentData { 'data' : string, 'timestamp' : bigint }
export interface ExperimentResult {
  'data' : string,
  'timestamp' : bigint,
  'success' : boolean,
}
export type Result = { 'Ok' : string } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : ExperimentData } |
  { 'Err' : string };
export interface _SERVICE {
  'compare_approaches' : ActorMethod<[], string>,
  'get_status_robust' : ActorMethod<[], Result>,
  'get_status_simple' : ActorMethod<[], string>,
  'greet_robust' : ActorMethod<[string], Result>,
  'greet_simple' : ActorMethod<[string], string>,
  'health' : ActorMethod<[], string>,
  'run_experiment_robust' : ActorMethod<[string], Result_1>,
  'run_experiment_simple' : ActorMethod<[string], ExperimentResult>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
