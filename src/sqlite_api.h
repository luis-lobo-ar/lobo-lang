#ifndef LOBO_SQLITE_ABI_V1_H
#define LOBO_SQLITE_ABI_V1_H
#ifndef LOBO_NATIVE_ABI_V1_H
#error "SQLite extension v1 requires the native ABI v1 result/buffer layouts"
#endif
#ifdef __cplusplus
extern "C" {
#endif
uint64_t lobo_sqlite_capability_v1(void);
lobo_native_result_v1 lobo_sqlite_restrict_v1(uint64_t cap, int64_t mask);
lobo_native_result_v1 lobo_sqlite_open_v1(uint64_t cap);
lobo_native_result_v1 lobo_sqlite_close_v1(uint64_t db);
lobo_native_result_v1 lobo_sqlite_prepare_v1(uint64_t db, lobo_buffer sql);
lobo_native_result_v1 lobo_sqlite_bind_null_v1(uint64_t statement, int64_t index);
lobo_native_result_v1 lobo_sqlite_bind_i64_v1(uint64_t statement, int64_t index, int64_t value);
lobo_native_result_v1 lobo_sqlite_bind_f64_v1(uint64_t statement, int64_t index, double value);
lobo_native_result_v1 lobo_sqlite_bind_text_v1(uint64_t statement, int64_t index, lobo_buffer value);
lobo_native_result_v1 lobo_sqlite_bind_bytes_v1(uint64_t statement, int64_t index, lobo_buffer value);
lobo_native_result_v1 lobo_sqlite_step_v1(uint64_t handle);
lobo_native_result_v1 lobo_sqlite_column_count_v1(uint64_t handle);
lobo_native_result_v1 lobo_sqlite_finalize_v1(uint64_t handle);
lobo_native_result_v1 lobo_sqlite_begin_v1(uint64_t handle);
lobo_native_result_v1 lobo_sqlite_commit_v1(uint64_t handle);
lobo_native_result_v1 lobo_sqlite_rollback_v1(uint64_t handle);
lobo_native_result_v1 lobo_sqlite_changes_v1(uint64_t handle);
lobo_native_result_v1 lobo_sqlite_column_type_v1(uint64_t statement, int64_t column);
lobo_native_result_v1 lobo_sqlite_column_i64_v1(uint64_t statement, int64_t column);
lobo_native_result_v1 lobo_sqlite_column_f64_v1(uint64_t statement, int64_t column);
lobo_native_result_v1 lobo_sqlite_column_text_v1(uint64_t statement, int64_t column);
lobo_native_result_v1 lobo_sqlite_column_bytes_v1(uint64_t statement, int64_t column);
#ifdef __cplusplus
}
#endif
#endif
