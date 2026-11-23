/* Closed SQLite extension v1. The engine is trusted native code, not a sandbox. */
#include <math.h>
enum { LS_DENIED=1, LS_INPUT=2, LS_HANDLE=3, LS_KIND=4, LS_CLOSED=5,
       LS_BUSY=7, LS_CONSTRAINT=8, LS_TYPE=9, LS_IO=10, LS_LIMIT=11, LS_SQL=12, LS_STATE=13 };
enum { LS_FREE=0, LS_CAP=1, LS_DB=2, LS_STMT=3, LS_RETIRED=4 };
enum { LS_READY=0, LS_ROW=1, LS_DONE=2, LS_FAILED=3 };
typedef struct {
    uint64_t generation, parent, bound;
    void *pointer;
    unsigned kind, rights, state, rows;
    bool control, denied;
} lobo_sqlite_slot;
static lobo_sqlite_slot lobo_sqlite_slots[32];
static uint64_t lobo_sqlite_root=0, lobo_sqlite_calls=4096, lobo_sqlite_bytes=1048576;
static uint64_t lobo_sqlite_steps=100000, lobo_sqlite_memory=8388608;
static unsigned lobo_sqlite_handle_limit=32, lobo_sqlite_row_limit=128, lobo_sqlite_options=0;
static unsigned lobo_sqlite_grant=1;
static const char *lobo_sqlite_file=NULL;
static bool lobo_sqlite_started=false;
static lobo_native_result_v1 lobo_sqlite_error(int64_t code) {
    lobo_native_result_v1 result={0};result.error=code;return result;
}
static lobo_native_result_v1 lobo_sqlite_integer(int64_t value) {
    lobo_native_result_v1 result={0};result.value.integer=value;return result;
}
static lobo_native_result_v1 lobo_sqlite_boolean(bool value) {
    lobo_native_result_v1 result={0};result.value.boolean=value;return result;
}
static int64_t lobo_sqlite_code(int code) {
    int64_t category;
    switch(code & 255) {
        case SQLITE_OK: case SQLITE_ROW: case SQLITE_DONE:return 0;
        case SQLITE_PERM: case SQLITE_AUTH: case SQLITE_READONLY:category=LS_DENIED;break;
        case SQLITE_BUSY: case SQLITE_LOCKED:category=LS_BUSY;break;
        case SQLITE_CONSTRAINT:category=LS_CONSTRAINT;break;
        case SQLITE_NOMEM: case SQLITE_TOOBIG: case SQLITE_FULL: case SQLITE_INTERRUPT:category=LS_LIMIT;break;
        case SQLITE_MISMATCH: case SQLITE_RANGE:category=LS_TYPE;break;
        case SQLITE_CANTOPEN: case SQLITE_IOERR: case SQLITE_CORRUPT: case SQLITE_NOTADB:category=LS_IO;break;
        default:category=LS_SQL;break;
    }
    return (int64_t)(unsigned)code*65536+category;
}
static bool lobo_sqlite_number(const char *value,uint64_t maximum,uint64_t *out) {
    if(!value || !*value || (value[0]=='0' && value[1])) return false;
    uint64_t n=0;
    for(const char *p=value;*p;++p) {
        if(*p<'0' || *p>'9' || n>maximum/10) return false;
        n=n*10+(unsigned)(*p-'0');if(n>maximum) return false;
    }
    *out=n;return true;
}
/* The launch grant names one absolute path. Parent directories must be trusted against replacement.
   URI names, dot components and symlinks/reparse points are rejected. This is not a custom secure VFS. */
static bool lobo_sqlite_path(const char *path) {
    size_t length=strlen(path);
    if(!length || length>1023 || !lobo_utf8((const unsigned char *)path,length,false)) return false;
    char partial[1024];memcpy(partial,path,length+1);
#if defined(_WIN64)
    if(length<4 || !((path[0]>='A'&&path[0]<='Z') || (path[0]>='a'&&path[0]<='z')) || path[1]!=':' || (path[2]!='\\'&&path[2]!='/')) return false;
    size_t begin=3;
#else
    if(path[0]!='/') return false;
    size_t begin=1;
#endif
    for(size_t i=begin;i<=length;++i) {
        bool separator=i==length || path[i]=='/'
#if defined(_WIN64)
            || path[i]=='\\'
#endif
            ;
        if(!separator) continue;
        size_t n=i-begin;
        if(!n || (n==1 && path[begin]=='.') || (n==2 && path[begin]=='.'&&path[begin+1]=='.')) return false;
        char saved=partial[i];partial[i]=0;
#if defined(_WIN64)
        wchar_t wide[1024];int count=MultiByteToWideChar(CP_UTF8,MB_ERR_INVALID_CHARS,partial,-1,wide,1024);
        if(!count) return false;
        DWORD attrs=GetFileAttributesW(wide);
        if(attrs==INVALID_FILE_ATTRIBUTES) {
            DWORD error=GetLastError();if(i!=length || error!=ERROR_FILE_NOT_FOUND) return false;
        } else if((attrs&FILE_ATTRIBUTE_REPARSE_POINT) || ((attrs&FILE_ATTRIBUTE_DIRECTORY)!=0)!=(i!=length)) return false;
#else
        struct stat info;
        if(lstat(partial,&info)) {if(i!=length || errno!=ENOENT) return false;}
        else if(S_ISLNK(info.st_mode) || (i==length ? !S_ISREG(info.st_mode) : !S_ISDIR(info.st_mode))) return false;
#endif
        partial[i]=saved;begin=i+1;
    }
    return true;
}
static bool lobo_sqlite_option(const char *name,const char *value) {
    unsigned bit=0;uint64_t number=0;
    if(!strcmp(name,"--sqlite-file")) bit=1;
    else if(!strcmp(name,"--sqlite-mode")) bit=2;
    else if(!strcmp(name,"--sqlite-calls")) bit=4;
    else if(!strcmp(name,"--sqlite-bytes")) bit=8;
    else if(!strcmp(name,"--sqlite-steps")) bit=16;
    else if(!strcmp(name,"--sqlite-memory")) bit=32;
    else if(!strcmp(name,"--sqlite-handles")) bit=64;
    else if(!strcmp(name,"--sqlite-rows")) bit=128;
    else return false;
    if(lobo_sqlite_options&bit) return false;
    if(bit==1) {if(!lobo_sqlite_path(value)) return false;lobo_sqlite_file=value;}
    else if(bit==2) {
        if(!strcmp(value,"read")) lobo_sqlite_grant=1;
        else if(!strcmp(value,"write")) lobo_sqlite_grant=3;
        else if(!strcmp(value,"schema")) lobo_sqlite_grant=7;
        else return false;
    } else {
        uint64_t maximum=bit==4?65536:bit==8?1048576:bit==16?10000000:bit==32?67108864:bit==64?32:1024;
        if(!lobo_sqlite_number(value,maximum,&number) || (bit!=8 && !number) || (bit==32 && number<1048576)) return false;
        if(bit==4) lobo_sqlite_calls=number;else if(bit==8) lobo_sqlite_bytes=number;
        else if(bit==16) lobo_sqlite_steps=number;else if(bit==32) lobo_sqlite_memory=number;
        else if(bit==64) lobo_sqlite_handle_limit=(unsigned)number;else lobo_sqlite_row_limit=(unsigned)number;
    }
    lobo_sqlite_options|=bit;return true;
}
static bool lobo_native_option(const char *name,const char *value) {
    return !strncmp(name,"--sqlite-",9) ? lobo_sqlite_option(name,value) : lobo_native_network_option(name,value);
}
static int64_t lobo_sqlite_lookup(uint64_t token,unsigned kind,lobo_sqlite_slot **out) {
    unsigned index=(unsigned)(token&255);uint64_t generation=(token>>8)&UINT64_C(0x0000ffffffffffff);
    if((token>>56)!=0x53 || !index || index>32 || !generation) return LS_HANDLE;
    lobo_sqlite_slot *slot=&lobo_sqlite_slots[index-1];
    if(slot->kind==LS_FREE && slot->generation==generation+1) return LS_CLOSED;
    if(slot->generation!=generation || slot->kind==LS_FREE || slot->kind==LS_RETIRED) return LS_HANDLE;
    if(slot->kind!=kind) return LS_KIND;
    *out=slot;return 0;
}
static int64_t lobo_sqlite_call(uint64_t token,unsigned kind,lobo_sqlite_slot **out) {
    int64_t error=lobo_sqlite_lookup(token,kind,out);if(error) return error;
    if(!lobo_sqlite_calls) return LS_LIMIT;
    --lobo_sqlite_calls;return 0;
}
static int64_t lobo_sqlite_insert(unsigned kind,void *pointer,unsigned rights,uint64_t parent,uint64_t *token) {
    unsigned live=0;
    for(unsigned i=0;i<32;++i) if(lobo_sqlite_slots[i].kind!=LS_FREE && lobo_sqlite_slots[i].kind!=LS_RETIRED) ++live;
    if(live>=lobo_sqlite_handle_limit) return LS_LIMIT;
    for(unsigned i=0;i<32;++i) {
        lobo_sqlite_slot *slot=&lobo_sqlite_slots[i];if(slot->kind!=LS_FREE) continue;
        if(!slot->generation) slot->generation=1;
        slot->kind=kind;slot->pointer=pointer;slot->rights=rights;slot->parent=parent;
        slot->state=LS_READY;slot->rows=0;slot->bound=0;slot->control=false;slot->denied=false;
        *token=UINT64_C(0x5300000000000000)|(slot->generation<<8)|(i+1);return 0;
    }
    return LS_LIMIT;
}
static void lobo_sqlite_release(lobo_sqlite_slot *slot) {
    slot->pointer=NULL;slot->parent=0;slot->rights=0;slot->bound=0;slot->control=false;slot->denied=false;
    if(slot->generation==UINT64_C(0x0000ffffffffffff)) slot->kind=LS_RETIRED;
    else {++slot->generation;slot->kind=LS_FREE;}
}
static int64_t lobo_sqlite_reserve(uint64_t amount) {
    if(amount>lobo_sqlite_bytes) return LS_LIMIT;
    lobo_sqlite_bytes-=amount;return 0;
}
static int lobo_sqlite_progress(void *context) {
    (void)context;
    if(!lobo_sqlite_steps) return 1;
    --lobo_sqlite_steps;return 0;
}
static int lobo_sqlite_deny(lobo_sqlite_slot *slot) {slot->denied=true;return SQLITE_DENY;}
static int64_t lobo_sqlite_engine_code(lobo_sqlite_slot *slot,int code) {
    return slot->denied?(int64_t)(unsigned)code*65536+LS_DENIED:lobo_sqlite_code(code);
}
static int lobo_sqlite_authorize(void *context,int op,const char *first,const char *second,const char *database,const char *trigger) {
    (void)first;(void)trigger;lobo_sqlite_slot *slot=context;
    if(database && strcmp(database,"main")) return lobo_sqlite_deny(slot);
    switch(op) {
        case SQLITE_SELECT: case SQLITE_READ: case SQLITE_RECURSIVE:return (slot->rights&1)?SQLITE_OK:lobo_sqlite_deny(slot);
        case SQLITE_INSERT: case SQLITE_UPDATE: case SQLITE_DELETE:return (slot->rights&2)?SQLITE_OK:lobo_sqlite_deny(slot);
        case SQLITE_CREATE_TABLE: case SQLITE_CREATE_INDEX: case SQLITE_DROP_TABLE: case SQLITE_DROP_INDEX:
        case SQLITE_REINDEX:return (slot->rights&4)?SQLITE_OK:lobo_sqlite_deny(slot);
        case SQLITE_TRANSACTION:return slot->control?SQLITE_OK:lobo_sqlite_deny(slot);
        case SQLITE_FUNCTION: {
            const char *allowed[]={"count","length","typeof","coalesce","sum","min","max","abs","substr","lower","upper","hex","round"};
            for(size_t i=0;second && i<sizeof(allowed)/sizeof(*allowed);++i) if(!sqlite3_stricmp(second,allowed[i])) return SQLITE_OK;
            return lobo_sqlite_deny(slot);
        }
        default:return lobo_sqlite_deny(slot);
    }
}
uint64_t lobo_sqlite_capability_v1(void) {
    if(lobo_depth!=1) lobo_fail("invocation.native.authority");
    if(lobo_sqlite_root) return lobo_sqlite_root;
    if(lobo_sqlite_insert(LS_CAP,NULL,lobo_sqlite_file?lobo_sqlite_grant:0,0,&lobo_sqlite_root)) lobo_fail("resource.native.exhausted");
    return lobo_sqlite_root;
}
lobo_native_result_v1 lobo_sqlite_restrict_v1(uint64_t cap,int64_t mask) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_call(cap,LS_CAP,&slot);
    if(error) return lobo_sqlite_error(error);
    if(mask<0 || mask>7) return lobo_sqlite_error(LS_INPUT);
    if(((unsigned)mask&slot->rights)!=(unsigned)mask) return lobo_sqlite_error(LS_DENIED);
    lobo_native_result_v1 result={0};error=lobo_sqlite_insert(LS_CAP,NULL,(unsigned)mask,cap,&result.value.handle);
    return error?lobo_sqlite_error(error):result;
}
lobo_native_result_v1 lobo_sqlite_open_v1(uint64_t cap) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_call(cap,LS_CAP,&slot);
    if(error) return lobo_sqlite_error(error);
    if(!(slot->rights&1) || !lobo_sqlite_file) return lobo_sqlite_error(LS_DENIED);
    if(!lobo_sqlite_path(lobo_sqlite_file)) return lobo_sqlite_error(LS_DENIED);
    const char *suffixes[]={"-journal","-wal","-shm"};
    for(unsigned i=0;i<3;++i) {
        char side[1024];int n=snprintf(side,sizeof(side),"%s%s",lobo_sqlite_file,suffixes[i]);
        if(n<0 || (size_t)n>=sizeof(side) || !lobo_sqlite_path(side)) return lobo_sqlite_error(LS_DENIED);
    }
    if(!lobo_sqlite_started) {
        if(sqlite3_initialize()!=SQLITE_OK) return lobo_sqlite_error(LS_LIMIT);
        lobo_sqlite_started=true;sqlite3_hard_heap_limit64((sqlite3_int64)lobo_sqlite_memory);
    }
    int flags=(slot->rights&2)?SQLITE_OPEN_READWRITE:SQLITE_OPEN_READONLY;
    if(slot->rights&4) flags|=SQLITE_OPEN_CREATE;
    sqlite3 *db=NULL;int code=sqlite3_open_v2(lobo_sqlite_file,&db,flags|SQLITE_OPEN_NOFOLLOW|SQLITE_OPEN_EXRESCODE,NULL);
    if(code!=SQLITE_OK) {if(db) sqlite3_close(db);return lobo_sqlite_error(lobo_sqlite_code(code));}
    /* No SQL callback can fault into Lobo before ownership is registered. */
    lobo_native_result_v1 result={0};error=lobo_sqlite_insert(LS_DB,db,slot->rights,cap,&result.value.handle);
    if(error) {sqlite3_close(db);return lobo_sqlite_error(error);}
    lobo_sqlite_slot *owner=&lobo_sqlite_slots[(result.value.handle&255)-1];
    int ignored=0;
    code=sqlite3_db_config(db,SQLITE_DBCONFIG_DEFENSIVE,1,&ignored);
    if(code==SQLITE_OK) code=sqlite3_db_config(db,SQLITE_DBCONFIG_TRUSTED_SCHEMA,0,&ignored);
    if(code==SQLITE_OK) code=sqlite3_exec(db,"PRAGMA temp_store=MEMORY; PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA mmap_size=0",NULL,NULL,NULL);
    if(code!=SQLITE_OK) {sqlite3_close(db);lobo_sqlite_release(owner);return lobo_sqlite_error(lobo_sqlite_code(code));}
    sqlite3_limit(db,SQLITE_LIMIT_LENGTH,65536);sqlite3_limit(db,SQLITE_LIMIT_SQL_LENGTH,65536);
    sqlite3_limit(db,SQLITE_LIMIT_COLUMN,32);sqlite3_limit(db,SQLITE_LIMIT_VARIABLE_NUMBER,32);
    sqlite3_limit(db,SQLITE_LIMIT_ATTACHED,0);sqlite3_limit(db,SQLITE_LIMIT_EXPR_DEPTH,32);
    sqlite3_limit(db,SQLITE_LIMIT_COMPOUND_SELECT,8);sqlite3_limit(db,SQLITE_LIMIT_TRIGGER_DEPTH,0);
    sqlite3_limit(db,SQLITE_LIMIT_VDBE_OP,25000);
    sqlite3_busy_timeout(db,0);sqlite3_progress_handler(db,1,lobo_sqlite_progress,NULL);
    sqlite3_set_authorizer(db,lobo_sqlite_authorize,owner);
    return result;
}
static void lobo_sqlite_finalize_owned(lobo_sqlite_slot *slot) {
    (void)sqlite3_finalize(slot->pointer);lobo_sqlite_release(slot);
}
static void lobo_sqlite_discard_db(lobo_sqlite_slot *slot) {
    sqlite3 *db=slot->pointer;
    sqlite3_progress_handler(db,0,NULL,NULL);slot->control=true;
    if(!sqlite3_get_autocommit(db)) (void)sqlite3_exec(db,"ROLLBACK",NULL,NULL,NULL);
    (void)sqlite3_close_v2(db);lobo_sqlite_release(slot);
}
static void lobo_sqlite_cleanup(void) {
    for(unsigned i=0;i<32;++i) if(lobo_sqlite_slots[i].kind==LS_STMT) lobo_sqlite_finalize_owned(&lobo_sqlite_slots[i]);
    for(unsigned i=0;i<32;++i) if(lobo_sqlite_slots[i].kind==LS_DB) lobo_sqlite_discard_db(&lobo_sqlite_slots[i]);
    for(unsigned i=0;i<32;++i) if(lobo_sqlite_slots[i].kind==LS_CAP) lobo_sqlite_release(&lobo_sqlite_slots[i]);
    if(lobo_sqlite_started) {(void)sqlite3_shutdown();lobo_sqlite_started=false;}
}
static void lobo_native_cleanup(void) {lobo_sqlite_cleanup();lobo_native_network_cleanup();}
lobo_native_result_v1 lobo_sqlite_close_v1(uint64_t token) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_lookup(token,LS_DB,&slot);
    if(error) return lobo_sqlite_error(error);
    for(unsigned i=0;i<32;++i) if(lobo_sqlite_slots[i].kind==LS_STMT && lobo_sqlite_slots[i].parent==token) lobo_sqlite_finalize_owned(&lobo_sqlite_slots[i]);
    sqlite3 *db=slot->pointer;sqlite3_progress_handler(db,0,NULL,NULL);slot->control=true;
    if(!sqlite3_get_autocommit(db)) (void)sqlite3_exec(db,"ROLLBACK",NULL,NULL,NULL);
    int code=sqlite3_close(db);
    if(code!=SQLITE_OK) {slot->control=false;slot->denied=false;sqlite3_progress_handler(db,1,lobo_sqlite_progress,NULL);return lobo_sqlite_error(lobo_sqlite_code(code));}
    lobo_sqlite_release(slot);return lobo_sqlite_boolean(true);
}
lobo_native_result_v1 lobo_sqlite_prepare_v1(uint64_t token,lobo_buffer sql) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_call(token,LS_DB,&slot);
    if(error) return lobo_sqlite_error(error);
    if(!sql.length || sql.length>65536 || !sql.data || memchr(sql.data,0,(size_t)sql.length) || !lobo_utf8(sql.data,sql.length,false)) return lobo_sqlite_error(LS_INPUT);
    if((error=lobo_sqlite_reserve(sql.length))) return lobo_sqlite_error(error);
    sqlite3_stmt *statement=NULL;const char *tail=NULL;
    slot->denied=false;
    int code=sqlite3_prepare_v3(slot->pointer,(const char *)sql.data,(int)sql.length,0,&statement,&tail);
    if(code!=SQLITE_OK) {if(statement) sqlite3_finalize(statement);return lobo_sqlite_error(lobo_sqlite_engine_code(slot,code));}
    if(!statement) return lobo_sqlite_error(LS_INPUT);
    if(sqlite3_column_count(statement)>0 && !sqlite3_stmt_readonly(statement)) {sqlite3_finalize(statement);return lobo_sqlite_error(LS_DENIED);}
    const char *end=(const char *)sql.data+sql.length;
    for(const char *p=tail;p<end;++p) if(*p!=' ' && *p!='\t' && *p!='\r' && *p!='\n') {sqlite3_finalize(statement);return lobo_sqlite_error(LS_INPUT);}
    lobo_native_result_v1 result={0};error=lobo_sqlite_insert(LS_STMT,statement,slot->rights,token,&result.value.handle);
    if(error) {sqlite3_finalize(statement);return lobo_sqlite_error(error);}
    return result;
}
static int64_t lobo_sqlite_bind_start(uint64_t token,int64_t index,lobo_sqlite_slot **slot) {
    int64_t error=lobo_sqlite_call(token,LS_STMT,slot);if(error) return error;
    if((*slot)->state!=LS_READY) return LS_STATE;
    if(index<1 || index>sqlite3_bind_parameter_count((*slot)->pointer)) return LS_INPUT;
    return 0;
}
static lobo_native_result_v1 lobo_sqlite_bound(lobo_sqlite_slot *slot,int64_t index,int code) {
    if(code!=SQLITE_OK) return lobo_sqlite_error(lobo_sqlite_code(code));
    slot->bound|=UINT64_C(1)<<(index-1);return lobo_sqlite_boolean(true);
}
lobo_native_result_v1 lobo_sqlite_bind_null_v1(uint64_t token,int64_t index) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_bind_start(token,index,&slot);
    return error?lobo_sqlite_error(error):lobo_sqlite_bound(slot,index,sqlite3_bind_null(slot->pointer,(int)index));
}
lobo_native_result_v1 lobo_sqlite_bind_i64_v1(uint64_t token,int64_t index,int64_t value) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_bind_start(token,index,&slot);
    return error?lobo_sqlite_error(error):lobo_sqlite_bound(slot,index,sqlite3_bind_int64(slot->pointer,(int)index,value));
}
lobo_native_result_v1 lobo_sqlite_bind_f64_v1(uint64_t token,int64_t index,double value) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_bind_start(token,index,&slot);
    if(error) return lobo_sqlite_error(error);
    if(!isfinite(value)) return lobo_sqlite_error(LS_TYPE);
    return lobo_sqlite_bound(slot,index,sqlite3_bind_double(slot->pointer,(int)index,value));
}
static lobo_native_result_v1 lobo_sqlite_bind_buffer(uint64_t token,int64_t index,lobo_buffer value,bool text) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_bind_start(token,index,&slot);
    if(error) return lobo_sqlite_error(error);
    if(value.length>65536 || (value.length&&!value.data) || (text&&!lobo_utf8(value.data,value.length,false))) return lobo_sqlite_error(LS_INPUT);
    if((error=lobo_sqlite_reserve(value.length))) return lobo_sqlite_error(error);
    const void *data=value.length?value.data:(const unsigned char *)"";
    int code=text?sqlite3_bind_text64(slot->pointer,(int)index,data,value.length,SQLITE_TRANSIENT,SQLITE_UTF8):sqlite3_bind_blob64(slot->pointer,(int)index,data,value.length,SQLITE_TRANSIENT);
    return lobo_sqlite_bound(slot,index,code);
}
lobo_native_result_v1 lobo_sqlite_bind_text_v1(uint64_t t,int64_t i,lobo_buffer v) {return lobo_sqlite_bind_buffer(t,i,v,true);}
lobo_native_result_v1 lobo_sqlite_bind_bytes_v1(uint64_t t,int64_t i,lobo_buffer v) {return lobo_sqlite_bind_buffer(t,i,v,false);}
lobo_native_result_v1 lobo_sqlite_step_v1(uint64_t token) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_call(token,LS_STMT,&slot);
    if(error) return lobo_sqlite_error(error);
    if(slot->state!=LS_READY && slot->state!=LS_ROW) return lobo_sqlite_error(LS_STATE);
    unsigned parameters=(unsigned)sqlite3_bind_parameter_count(slot->pointer);
    if(slot->bound!=((UINT64_C(1)<<parameters)-1)) return lobo_sqlite_error(LS_INPUT);
    lobo_sqlite_slot *owner;
    if(lobo_sqlite_lookup(slot->parent,LS_DB,&owner)) return lobo_sqlite_error(LS_HANDLE);
    owner->denied=false;
    int code=sqlite3_step(slot->pointer);
    if(code==SQLITE_ROW) {
        if(slot->rows>=lobo_sqlite_row_limit) {slot->state=LS_FAILED;return lobo_sqlite_error(LS_LIMIT);}
        ++slot->rows;slot->state=LS_ROW;return lobo_sqlite_boolean(true);
    }
    slot->state=code==SQLITE_DONE?LS_DONE:LS_FAILED;
    return code==SQLITE_DONE?lobo_sqlite_boolean(false):lobo_sqlite_error(lobo_sqlite_engine_code(owner,code));
}
lobo_native_result_v1 lobo_sqlite_column_count_v1(uint64_t token) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_call(token,LS_STMT,&slot);
    return error?lobo_sqlite_error(error):lobo_sqlite_integer(sqlite3_column_count(slot->pointer));
}
static int64_t lobo_sqlite_column(uint64_t token,int64_t index,lobo_sqlite_slot **slot,int expected) {
    int64_t error=lobo_sqlite_call(token,LS_STMT,slot);if(error) return error;
    if((*slot)->state!=LS_ROW) return LS_STATE;
    if(index<0 || index>=sqlite3_column_count((*slot)->pointer)) return LS_INPUT;
    if(expected && sqlite3_column_type((*slot)->pointer,(int)index)!=expected) return LS_TYPE;
    return 0;
}
lobo_native_result_v1 lobo_sqlite_column_type_v1(uint64_t token,int64_t index) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_column(token,index,&slot,0);
    return error?lobo_sqlite_error(error):lobo_sqlite_integer(sqlite3_column_type(slot->pointer,(int)index));
}
lobo_native_result_v1 lobo_sqlite_column_i64_v1(uint64_t token,int64_t index) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_column(token,index,&slot,SQLITE_INTEGER);
    return error?lobo_sqlite_error(error):lobo_sqlite_integer(sqlite3_column_int64(slot->pointer,(int)index));
}
lobo_native_result_v1 lobo_sqlite_column_f64_v1(uint64_t token,int64_t index) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_column(token,index,&slot,SQLITE_FLOAT);
    if(error) return lobo_sqlite_error(error);
    double value=sqlite3_column_double(slot->pointer,(int)index);
    if(!isfinite(value)) return lobo_sqlite_error(LS_TYPE);
    lobo_native_result_v1 result={0};result.value.floating=value;return result;
}
static lobo_native_result_v1 lobo_sqlite_column_buffer(uint64_t token,int64_t index,bool text) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_column(token,index,&slot,text?SQLITE_TEXT:SQLITE_BLOB);
    if(error) return lobo_sqlite_error(error);
    const void *data=text?(const void *)sqlite3_column_text(slot->pointer,(int)index):sqlite3_column_blob(slot->pointer,(int)index);
    int count=sqlite3_column_bytes(slot->pointer,(int)index);
    if(count<0 || (!data && (count || sqlite3_errcode(sqlite3_db_handle(slot->pointer))==SQLITE_NOMEM))) return lobo_sqlite_error(LS_LIMIT);
    if(text && !lobo_utf8(data,(uint64_t)count,false)) return lobo_sqlite_error(LS_TYPE);
    if((error=lobo_sqlite_reserve((uint64_t)count))) return lobo_sqlite_error(error);
    lobo_spend((uint64_t)count);
    lobo_native_result_v1 result={0};result.value.buffer=lobo_buffer_new((uint64_t)count);
    if(count) memcpy((void *)result.value.buffer.data,data,(size_t)count);
    return result;
}
lobo_native_result_v1 lobo_sqlite_column_text_v1(uint64_t t,int64_t i) {return lobo_sqlite_column_buffer(t,i,true);}
lobo_native_result_v1 lobo_sqlite_column_bytes_v1(uint64_t t,int64_t i) {return lobo_sqlite_column_buffer(t,i,false);}
lobo_native_result_v1 lobo_sqlite_finalize_v1(uint64_t token) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_lookup(token,LS_STMT,&slot);
    if(error) return lobo_sqlite_error(error);
    /* Execution failures were reported by step. Finalization still consumes the handle. */
    lobo_sqlite_finalize_owned(slot);return lobo_sqlite_boolean(true);
}
static lobo_native_result_v1 lobo_sqlite_transaction(uint64_t token,int action) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_call(token,LS_DB,&slot);
    if(error) return lobo_sqlite_error(error);
    bool active=!sqlite3_get_autocommit(slot->pointer);
    if((action==0)==active) return lobo_sqlite_error(LS_STATE);
    for(unsigned i=0;i<32;++i) if(lobo_sqlite_slots[i].kind==LS_STMT && lobo_sqlite_slots[i].parent==token) return lobo_sqlite_error(LS_BUSY);
    slot->control=true;
    int code=sqlite3_exec(slot->pointer,action==0?"BEGIN":action==1?"COMMIT":"ROLLBACK",NULL,NULL,NULL);
    slot->control=false;slot->denied=false;
    return code==SQLITE_OK?lobo_sqlite_boolean(true):lobo_sqlite_error(lobo_sqlite_code(code));
}
lobo_native_result_v1 lobo_sqlite_begin_v1(uint64_t t) {return lobo_sqlite_transaction(t,0);}
lobo_native_result_v1 lobo_sqlite_commit_v1(uint64_t t) {return lobo_sqlite_transaction(t,1);}
lobo_native_result_v1 lobo_sqlite_rollback_v1(uint64_t t) {return lobo_sqlite_transaction(t,2);}
lobo_native_result_v1 lobo_sqlite_changes_v1(uint64_t token) {
    lobo_sqlite_slot *slot;int64_t error=lobo_sqlite_call(token,LS_DB,&slot);
    return error?lobo_sqlite_error(error):lobo_sqlite_integer(sqlite3_changes64(slot->pointer));
}
