/* Private LLVM value/context ABI. Application bodies are emitted directly as LLVM IR. */
struct lobo_ir_context { const char *subject, *phase, *contract; };
void lobo_ir_enter(void *storage, const char *id) {
    struct lobo_ir_context *saved = storage;
    *saved = (struct lobo_ir_context){lobo_subject, lobo_phase, lobo_contract_phase};
    lobo_subject = id; lobo_phase = "execute";
    lobo_tick();
    if (lobo_depth >= 32) lobo_fail("resource.stack.exhausted");
    ++lobo_depth;
}
void lobo_ir_leave(const void *storage) {
    const struct lobo_ir_context *saved = storage;
    --lobo_depth; lobo_subject = saved->subject;
    lobo_phase = saved->phase; lobo_contract_phase = saved->contract;
}
void lobo_ir_phase(const void *storage, int phase) {
    const struct lobo_ir_context *saved = storage;
    lobo_contract_phase = saved->contract;
    lobo_phase = phase == 1 ? "precondition" : phase == 2 ? "postcondition" : "execute";
    if (phase && !lobo_contract_phase) lobo_contract_phase = lobo_phase;
}
void lobo_ir_tick(void) { lobo_tick(); }
_Noreturn void lobo_ir_fail(const char *code) { lobo_fail(code); }
