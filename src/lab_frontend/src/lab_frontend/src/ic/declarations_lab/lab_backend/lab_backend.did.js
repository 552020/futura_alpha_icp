export const idlFactory = ({ IDL }) => {
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const ExperimentData = IDL.Record({
    'data' : IDL.Text,
    'timestamp' : IDL.Nat64,
  });
  const Result_1 = IDL.Variant({ 'Ok' : ExperimentData, 'Err' : IDL.Text });
  const ExperimentResult = IDL.Record({
    'data' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'success' : IDL.Bool,
  });
  return IDL.Service({
    'compare_approaches' : IDL.Func([], [IDL.Text], ['query']),
    'get_status_robust' : IDL.Func([], [Result], ['query']),
    'get_status_simple' : IDL.Func([], [IDL.Text], ['query']),
    'greet_robust' : IDL.Func([IDL.Text], [Result], ['query']),
    'greet_simple' : IDL.Func([IDL.Text], [IDL.Text], ['query']),
    'health' : IDL.Func([], [IDL.Text], ['query']),
    'run_experiment_robust' : IDL.Func([IDL.Text], [Result_1], []),
    'run_experiment_simple' : IDL.Func([IDL.Text], [ExperimentResult], []),
  });
};
export const init = ({ IDL }) => { return []; };
