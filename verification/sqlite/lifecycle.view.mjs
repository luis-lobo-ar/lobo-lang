export const title = 'SQLite connection, statement and transaction lifecycle';
export const state = v => {
  if (v.stored !== v.committed) return 'Uncommitted write persisted';
  if (v.badCommit) return 'Commit accepted without a transaction';
  if (v.rights.some(right => !v.ceiling.includes(right))) return 'Permissions widened';
  if (v.phase === 'exited') return v.cursor === 'none' ? 'Invocation cleaned up' : 'Statement leaked at exit';
  if (!v.connected) return 'No connection';
  if (v.active && v.pending > 0) return 'Transaction has a pending write';
  if (v.cursor !== 'none') return `Statement ${v.cursor}`;
  return v.active ? 'Transaction open' : 'Connection open';
};
export const step = action => ({Open:'Open database',Prepare:'Prepare statement',ReadRow:'Read a row',DoneStatement:'Finish statement',Finalize:'Finalize statement',Begin:'Begin transaction',Write:'Execute write',Commit:'Commit transaction',Rollback:'Roll back transaction',Derive:'Restrict permissions',Exit:'Return or fault and clean up',Finished:'Finished'}[action] ?? action);
