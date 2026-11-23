#include "../../src/sqlite_profile.h"
#if defined(LOBO_SQLITE_EXTERNAL)
#include "../../vendor/sqlite/sqlite3.h"
#else
#include "../../vendor/sqlite/sqlite3.c"
#endif
#include "../../src/native_runtime.c"
#include "../../src/native_api.h"
#include "../../src/native_buffers.c"
#include "../../src/native_floats.c"
static bool lobo_native_network_option(const char *name,const char *value) {(void)name;(void)value;return false;}
static void lobo_native_network_cleanup(void) {}
#if defined(__linux__)
#include <sys/stat.h>
#include <unistd.h>
#endif
#ifndef LOBO_SQLITE_PROVIDER
#define LOBO_SQLITE_PROVIDER "../../src/native_sqlite.c"
#endif
#include "../../src/sqlite_api.h"
#include LOBO_SQLITE_PROVIDER
#define CHECK(expression) do {if(!(expression)) {fprintf(stderr,"sqlite check failed at %d: %s\n",__LINE__,#expression);exit(23);}} while(0)
static lobo_buffer text(const char *value) {return (lobo_buffer){(const unsigned char *)value,strlen(value)};}
static uint64_t handle(lobo_native_result_v1 result) {if(result.error) fprintf(stderr,"unexpected sqlite error %lld\n",(long long)result.error);CHECK(!result.error);return result.value.handle;}
static bool boolean(lobo_native_result_v1 result) {CHECK(!result.error);return result.value.boolean;}
static int64_t integer(lobo_native_result_v1 result) {CHECK(!result.error);return result.value.integer;}
static int64_t category(lobo_native_result_v1 result) {return result.error%65536;}
static uint64_t prepare(uint64_t db,const char *sql) {return handle(lobo_sqlite_prepare_v1(db,text(sql)));}
static void execute(uint64_t db,const char *sql) {
    uint64_t statement=prepare(db,sql);CHECK(!boolean(lobo_sqlite_step_v1(statement)));CHECK(boolean(lobo_sqlite_finalize_v1(statement)));
}
static int64_t scalar(uint64_t db,const char *sql) {
    uint64_t statement=prepare(db,sql);CHECK(boolean(lobo_sqlite_step_v1(statement)));
    int64_t result=integer(lobo_sqlite_column_i64_v1(statement,0));CHECK(!boolean(lobo_sqlite_step_v1(statement)));
    CHECK(boolean(lobo_sqlite_finalize_v1(statement)));return result;
}
static int64_t oracle(const char *path) {
    sqlite3 *db=NULL;sqlite3_stmt *statement=NULL;
    CHECK(sqlite3_open_v2(path,&db,SQLITE_OPEN_READONLY,NULL)==SQLITE_OK);
    CHECK(sqlite3_prepare_v2(db,"SELECT count(*) FROM users",-1,&statement,NULL)==SQLITE_OK);
    CHECK(sqlite3_step(statement)==SQLITE_ROW);int64_t count=sqlite3_column_int64(statement,0);
    CHECK(sqlite3_finalize(statement)==SQLITE_OK);CHECK(sqlite3_close(db)==SQLITE_OK);return count;
}
static const char *observer_file=NULL;
static void observe_rollback(void) {
    for(unsigned i=0;i<32;++i) CHECK(lobo_sqlite_slots[i].kind!=LS_DB && lobo_sqlite_slots[i].kind!=LS_STMT);
    sqlite3_hard_heap_limit64(0);CHECK(oracle(observer_file)==0);CHECK(sqlite3_shutdown()==SQLITE_OK);
    puts("sqlite-fault-cleanup-passed");
}
static void values(uint64_t db) {
    uint64_t statement=prepare(db,"SELECT ?1,?2,?3,?4,?5");
    const unsigned char content[]={'w',0,0xf0,0x9f,0x90,0xba};const unsigned char bytes[]={0,255,127,128};
    CHECK(category(lobo_sqlite_column_i64_v1(statement,0))==LS_STATE);
    CHECK(category(lobo_sqlite_step_v1(statement))==LS_INPUT);
    CHECK(boolean(lobo_sqlite_bind_i64_v1(statement,1,INT64_MAX)));
    CHECK(boolean(lobo_sqlite_bind_f64_v1(statement,2,1.25)));
    CHECK(boolean(lobo_sqlite_bind_text_v1(statement,3,(lobo_buffer){content,sizeof(content)})));
    CHECK(boolean(lobo_sqlite_bind_bytes_v1(statement,4,(lobo_buffer){bytes,sizeof(bytes)})));
    CHECK(boolean(lobo_sqlite_bind_null_v1(statement,5)));
    CHECK(boolean(lobo_sqlite_step_v1(statement)));CHECK(integer(lobo_sqlite_column_count_v1(statement))==5);
    CHECK(integer(lobo_sqlite_column_i64_v1(statement,0))==INT64_MAX);
    lobo_native_result_v1 floating=lobo_sqlite_column_f64_v1(statement,1);CHECK(!floating.error && floating.value.floating==1.25);
    lobo_native_result_v1 copied=lobo_sqlite_column_text_v1(statement,2);CHECK(!copied.error && copied.value.buffer.length==sizeof(content));
    CHECK(!memcmp(copied.value.buffer.data,content,sizeof(content)));
    lobo_native_result_v1 blob=lobo_sqlite_column_bytes_v1(statement,3);CHECK(!blob.error && !memcmp(blob.value.buffer.data,bytes,sizeof(bytes)));
    CHECK(integer(lobo_sqlite_column_type_v1(statement,4))==SQLITE_NULL);
    CHECK(category(lobo_sqlite_column_text_v1(statement,4))==LS_TYPE);
    CHECK(category(lobo_sqlite_column_f64_v1(statement,0))==LS_TYPE);
    CHECK(category(lobo_sqlite_column_type_v1(statement,5))==LS_INPUT);
    CHECK(category(lobo_sqlite_bind_i64_v1(statement,1,2))==LS_STATE);
    CHECK(!boolean(lobo_sqlite_step_v1(statement)));CHECK(category(lobo_sqlite_step_v1(statement))==LS_STATE);
    CHECK(boolean(lobo_sqlite_finalize_v1(statement)));
    CHECK(!memcmp(copied.value.buffer.data,content,sizeof(content)));CHECK(!memcmp(blob.value.buffer.data,bytes,sizeof(bytes)));
}
static void empty_and_invalid(uint64_t db) {
    uint64_t s=prepare(db,"SELECT ?1,?2");
    CHECK(boolean(lobo_sqlite_bind_text_v1(s,1,(lobo_buffer){NULL,0})));
    CHECK(boolean(lobo_sqlite_bind_bytes_v1(s,2,(lobo_buffer){NULL,0})));
    CHECK(boolean(lobo_sqlite_step_v1(s)));
    CHECK(integer(lobo_sqlite_column_type_v1(s,0))==SQLITE_TEXT);CHECK(integer(lobo_sqlite_column_type_v1(s,1))==SQLITE_BLOB);
    CHECK(lobo_sqlite_column_text_v1(s,0).value.buffer.length==0);CHECK(lobo_sqlite_column_bytes_v1(s,1).value.buffer.length==0);
    CHECK(boolean(lobo_sqlite_finalize_v1(s)));
    s=prepare(db,"SELECT ?1");const unsigned char invalid[]={255};
    CHECK(category(lobo_sqlite_bind_text_v1(s,1,(lobo_buffer){invalid,1}))==LS_INPUT);
    CHECK(category(lobo_sqlite_bind_f64_v1(s,1,INFINITY))==LS_TYPE);CHECK(category(lobo_sqlite_bind_i64_v1(s,0,1))==LS_INPUT);
    CHECK(boolean(lobo_sqlite_finalize_v1(s)));
    s=prepare(db,"SELECT CAST(X'ff' AS TEXT)");CHECK(boolean(lobo_sqlite_step_v1(s)));CHECK(category(lobo_sqlite_column_text_v1(s,0))==LS_TYPE);CHECK(boolean(lobo_sqlite_finalize_v1(s)));
    CHECK(category(lobo_sqlite_prepare_v1(db,(lobo_buffer){(const unsigned char *)"SELECT 1\0SELECT 2",17}))==LS_INPUT);
}
static void transactions(uint64_t db,const char *path) {
    execute(db,"CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT NOT NULL)");
    CHECK(category(lobo_sqlite_commit_v1(db))==LS_STATE);CHECK(category(lobo_sqlite_rollback_v1(db))==LS_STATE);
    CHECK(boolean(lobo_sqlite_begin_v1(db)));CHECK(category(lobo_sqlite_begin_v1(db))==LS_STATE);
    execute(db,"INSERT INTO users VALUES(1,'Ada')");CHECK(boolean(lobo_sqlite_commit_v1(db)));CHECK(oracle(path)==1);
    CHECK(boolean(lobo_sqlite_begin_v1(db)));execute(db,"INSERT INTO users VALUES(2,'Luis')");CHECK(boolean(lobo_sqlite_rollback_v1(db)));CHECK(oracle(path)==1);
    CHECK(boolean(lobo_sqlite_begin_v1(db)));uint64_t statement=prepare(db,"SELECT id FROM users");
    CHECK(category(lobo_sqlite_commit_v1(db))==LS_BUSY);CHECK(boolean(lobo_sqlite_finalize_v1(statement)));CHECK(boolean(lobo_sqlite_rollback_v1(db)));
    statement=prepare(db,"INSERT INTO users VALUES(1,'duplicate')");lobo_native_result_v1 failure=lobo_sqlite_step_v1(statement);
    CHECK(category(failure)==LS_CONSTRAINT);CHECK((failure.error/65536 & 255)==SQLITE_CONSTRAINT);
    CHECK(boolean(lobo_sqlite_finalize_v1(statement)));CHECK(oracle(path)==1);
    CHECK(boolean(lobo_sqlite_begin_v1(db)));execute(db,"INSERT INTO users VALUES(3,'rolled back on close')");
    CHECK(boolean(lobo_sqlite_close_v1(db)));CHECK(oracle(path)==1);
}
static void policy(uint64_t db,uint64_t cap) {
    execute(db,"CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT)");execute(db,"INSERT INTO users VALUES(1,'Ada')");
    CHECK(category(lobo_sqlite_prepare_v1(db,text("SELECT randomblob(1)")))==LS_DENIED);
    const char *denied[]={"ATTACH DATABASE ':memory:' AS other","PRAGMA writable_schema=ON","SELECT load_extension('x')","SELECT readfile('/etc/passwd')","CREATE TEMP TABLE nope(x)","CREATE VIEW nope AS SELECT 1","BEGIN","SAVEPOINT x","PRAGMA journal_mode=WAL","ALTER TABLE users ADD COLUMN unsafe TEXT","INSERT INTO users VALUES(2,'x') RETURNING id"};
    for(unsigned i=0;i<sizeof(denied)/sizeof(*denied);++i) {lobo_native_result_v1 r=lobo_sqlite_prepare_v1(db,text(denied[i]));if(!r.error) fprintf(stderr,"unexpected prepare acceptance: %s\n",denied[i]);CHECK(r.error);}
    /* VACUUM authorization is deferred by SQLite until step, before its ATTACH. */
    uint64_t vacuum=prepare(db,"VACUUM");CHECK(category(lobo_sqlite_step_v1(vacuum))==LS_DENIED);CHECK(boolean(lobo_sqlite_finalize_v1(vacuum)));
    CHECK(scalar(db,"SELECT count(*) FROM users")==1);
    CHECK(category(lobo_sqlite_prepare_v1(db,text("SELECT 1; SELECT 2")))==LS_INPUT);
    uint64_t readcap=handle(lobo_sqlite_restrict_v1(cap,1));CHECK(category(lobo_sqlite_restrict_v1(readcap,7))==LS_DENIED);
    uint64_t readonly=handle(lobo_sqlite_open_v1(readcap));CHECK(scalar(readonly,"SELECT count(*) FROM users")==1);
    CHECK(category(lobo_sqlite_prepare_v1(readonly,text("DELETE FROM users")))==LS_DENIED);
    CHECK(category(lobo_sqlite_prepare_v1(readonly,text("CREATE TABLE nope(x)")))==LS_DENIED);
    CHECK(boolean(lobo_sqlite_close_v1(readonly)));
    uint64_t empty=handle(lobo_sqlite_restrict_v1(cap,0));CHECK(category(lobo_sqlite_open_v1(empty))==LS_DENIED);
    CHECK(category(lobo_sqlite_open_v1(db))==LS_KIND);CHECK(category(lobo_sqlite_open_v1(257))==LS_HANDLE);
}
static void injection(uint64_t db) {
    execute(db,"CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT)");
    const char *payload="Robert'); DROP TABLE users;--";uint64_t s=prepare(db,"INSERT INTO users VALUES(?1,?2)");
    CHECK(boolean(lobo_sqlite_bind_i64_v1(s,1,1)));CHECK(boolean(lobo_sqlite_bind_text_v1(s,2,text(payload))));
    CHECK(!boolean(lobo_sqlite_step_v1(s)));CHECK(boolean(lobo_sqlite_finalize_v1(s)));CHECK(scalar(db,"SELECT count(*) FROM users")==1);
    s=prepare(db,"SELECT name FROM users WHERE id=?1");CHECK(boolean(lobo_sqlite_bind_i64_v1(s,1,1)));CHECK(boolean(lobo_sqlite_step_v1(s)));
    lobo_native_result_v1 value=lobo_sqlite_column_text_v1(s,0);CHECK(!value.error && value.value.buffer.length==strlen(payload));CHECK(!memcmp(value.value.buffer.data,payload,strlen(payload)));
    CHECK(boolean(lobo_sqlite_finalize_v1(s)));
}
static void budgets(uint64_t db) {
    uint64_t s=prepare(db,"SELECT 1 UNION ALL SELECT 2");lobo_sqlite_row_limit=1;
    CHECK(boolean(lobo_sqlite_step_v1(s)));CHECK(category(lobo_sqlite_step_v1(s))==LS_LIMIT);CHECK(boolean(lobo_sqlite_finalize_v1(s)));
    lobo_sqlite_bytes=0;CHECK(category(lobo_sqlite_prepare_v1(db,text("SELECT 1")))==LS_LIMIT);lobo_sqlite_bytes=1048576;
    s=prepare(db,"WITH RECURSIVE c(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM c WHERE x<1000000) SELECT sum(x) FROM c");
    lobo_sqlite_steps=10;CHECK(category(lobo_sqlite_step_v1(s))==LS_LIMIT);CHECK(boolean(lobo_sqlite_finalize_v1(s)));lobo_sqlite_steps=100000;
    uint64_t pending=prepare(db,"SELECT 1");
    lobo_sqlite_calls=0;CHECK(category(lobo_sqlite_prepare_v1(db,text("SELECT 1")))==LS_LIMIT);
    CHECK(boolean(lobo_sqlite_finalize_v1(pending)));CHECK(boolean(lobo_sqlite_close_v1(db)));
    lobo_native_cleanup();lobo_native_cleanup();
}
static void stale(uint64_t db) {
    uint64_t s=prepare(db,"SELECT 1");CHECK(boolean(lobo_sqlite_finalize_v1(s)));CHECK(category(lobo_sqlite_step_v1(s))==LS_CLOSED);
    uint64_t replacement=prepare(db,"SELECT 2");CHECK(replacement!=s);CHECK(category(lobo_sqlite_step_v1(s))==LS_HANDLE);
    CHECK(boolean(lobo_sqlite_close_v1(db)));CHECK(category(lobo_sqlite_step_v1(replacement))==LS_CLOSED);CHECK(category(lobo_sqlite_close_v1(db))==LS_CLOSED);
}
static void busy(uint64_t db,const char *path) {
    execute(db,"CREATE TABLE users(id INTEGER)");sqlite3 *other=NULL;
    CHECK(sqlite3_open(path,&other)==SQLITE_OK);CHECK(sqlite3_exec(other,"BEGIN EXCLUSIVE",NULL,NULL,NULL)==SQLITE_OK);
    lobo_native_result_v1 prepared=lobo_sqlite_prepare_v1(db,text("SELECT count(*) FROM users"));
    if(!prepared.error) {CHECK(category(lobo_sqlite_step_v1(prepared.value.handle))==LS_BUSY);CHECK(boolean(lobo_sqlite_finalize_v1(prepared.value.handle)));}
    else CHECK(category(prepared)==LS_BUSY);
    CHECK(sqlite3_exec(other,"ROLLBACK",NULL,NULL,NULL)==SQLITE_OK);CHECK(sqlite3_close(other)==SQLITE_OK);
}
static void handle_quota(uint64_t db) {
    lobo_sqlite_handle_limit=2;CHECK(category(lobo_sqlite_prepare_v1(db,text("SELECT 1")))==LS_LIMIT);
    lobo_sqlite_slot *slot=NULL;CHECK(!lobo_sqlite_lookup(db,LS_DB,&slot));CHECK(!sqlite3_next_stmt(slot->pointer,NULL));
}
static void generation(uint64_t db) {
    uint64_t s=prepare(db,"SELECT 1");lobo_sqlite_slot *slot=NULL;CHECK(!lobo_sqlite_lookup(s,LS_STMT,&slot));
    slot->generation=UINT64_C(0x0000ffffffffffff);uint64_t maximum=UINT64_C(0x53ffffffffffff00)|(s&255);
    CHECK(boolean(lobo_sqlite_finalize_v1(maximum)));CHECK(slot->kind==LS_RETIRED);
    uint64_t newer=prepare(db,"SELECT 2");CHECK((newer&255)!=(s&255));CHECK(category(lobo_sqlite_step_v1(s))==LS_HANDLE);
}
static void fault(uint64_t db,const char *path,bool heap) {
    execute(db,"CREATE TABLE users(id INTEGER)");CHECK(boolean(lobo_sqlite_begin_v1(db)));execute(db,"INSERT INTO users VALUES(1)");
    uint64_t s=prepare(db,"SELECT 'fault cleanup'");CHECK(boolean(lobo_sqlite_step_v1(s)));
    observer_file=path;lobo_sqlite_calls=1;
    if(heap) lobo_heap_cells=0;else lobo_fuel=0;
    (void)lobo_sqlite_column_text_v1(s,0);CHECK(false);
}
static void commit_busy(uint64_t db,const char *path) {
    execute(db,"CREATE TABLE users(id INTEGER)");execute(db,"INSERT INTO users VALUES(1)");
    sqlite3 *reader=NULL;sqlite3_stmt *cursor=NULL;
    CHECK(sqlite3_open(path,&reader)==SQLITE_OK);
    CHECK(sqlite3_prepare_v2(reader,"SELECT id FROM users",-1,&cursor,NULL)==SQLITE_OK);
    CHECK(sqlite3_step(cursor)==SQLITE_ROW);
    CHECK(boolean(lobo_sqlite_begin_v1(db)));execute(db,"INSERT INTO users VALUES(2)");
    CHECK(category(lobo_sqlite_commit_v1(db))==LS_BUSY);
    CHECK(boolean(lobo_sqlite_rollback_v1(db)));
    CHECK(sqlite3_finalize(cursor)==SQLITE_OK);CHECK(sqlite3_close(reader)==SQLITE_OK);
    CHECK(oracle(path)==1);
}
static void memory_failure(uint64_t db) {
    CHECK(sqlite3_hard_heap_limit64(-1)==1048576);
    uint64_t s=prepare(db,"SELECT ?1");
    sqlite3_hard_heap_limit64(sqlite3_memory_used()+1024);
    static const unsigned char data[65536]={1};
    lobo_native_result_v1 failure=lobo_sqlite_bind_bytes_v1(s,1,(lobo_buffer){data,sizeof(data)});
    CHECK(category(failure)==LS_LIMIT);CHECK((failure.error/65536 & 255)==SQLITE_NOMEM);
    CHECK(boolean(lobo_sqlite_finalize_v1(s)));CHECK(boolean(lobo_sqlite_close_v1(db)));
}
#if defined(__linux__)
static void side_symlink(uint64_t db,uint64_t cap,const char *path) {
    CHECK(boolean(lobo_sqlite_close_v1(db)));
    char side[1200],target[1200];CHECK(snprintf(side,sizeof(side),"%s-journal",path)>0);CHECK(snprintf(target,sizeof(target),"%s-other",path)>0);
    FILE *file=fopen(target,"wb");CHECK(file);CHECK(fwrite("keep",1,4,file)==4);CHECK(fclose(file)==0);
    CHECK(symlink(target,side)==0);CHECK(category(lobo_sqlite_open_v1(cap))==LS_DENIED);
    char bytes[4];file=fopen(target,"rb");CHECK(file);CHECK(fread(bytes,1,4,file)==4);CHECK(fclose(file)==0);CHECK(!memcmp(bytes,"keep",4));
    CHECK(unlink(side)==0);CHECK(unlink(target)==0);
}
#endif
int main(int argc,char **argv) {
    CHECK(argc==3);lobo_validating=false;lobo_depth=1;
    bool is_fault=!strcmp(argv[1],"fuel-cleanup") || !strcmp(argv[1],"heap-cleanup");
    if(is_fault) CHECK(atexit(observe_rollback)==0);
    CHECK(atexit(lobo_cleanup)==0);
    if(!strcmp(argv[1],"deny")) {uint64_t c=lobo_sqlite_capability_v1();CHECK(category(lobo_sqlite_open_v1(c))==LS_DENIED);puts("sqlite-adapter-case-passed");return 0;}
    if(!strcmp(argv[1],"options")) {
        CHECK(!lobo_native_option("--sqlite-file",":memory:"));CHECK(!lobo_native_option("--sqlite-file","file:/tmp/x"));CHECK(!lobo_native_option("--sqlite-file","relative.sqlite"));
        CHECK(!lobo_native_option("--sqlite-file","/tmp/../escape.sqlite"));CHECK(!lobo_native_option("--sqlite-mode","admin"));
        CHECK(!lobo_native_option("--sqlite-steps","0"));CHECK(!lobo_native_option("--sqlite-memory","1"));CHECK(!lobo_native_option("--sqlite-rows","01"));
        CHECK(lobo_native_option("--sqlite-rows","1"));CHECK(!lobo_native_option("--sqlite-rows","2"));puts("sqlite-adapter-case-passed");return 0;
    }
    CHECK(lobo_native_option("--sqlite-file",argv[2]));CHECK(lobo_native_option("--sqlite-mode","schema"));
    if(!strcmp(argv[1],"memory")) CHECK(lobo_native_option("--sqlite-memory","1048576"));
    uint64_t cap=lobo_sqlite_capability_v1();
    if(!strcmp(argv[1],"corrupt")) {
        FILE *file=fopen(argv[2],"wb");CHECK(file);CHECK(fwrite("not a database",1,14,file)==14);CHECK(fclose(file)==0);
        CHECK(category(lobo_sqlite_open_v1(cap))==LS_IO);
        char data[14];file=fopen(argv[2],"rb");CHECK(file);CHECK(fread(data,1,14,file)==14);CHECK(fclose(file)==0);CHECK(!memcmp(data,"not a database",14));
        puts("sqlite-adapter-case-passed");return 0;
    }
    if(!strcmp(argv[1],"authority")) {lobo_depth=2;(void)lobo_sqlite_capability_v1();CHECK(false);}
    uint64_t db=handle(lobo_sqlite_open_v1(cap));
    if(!strcmp(argv[1],"open")) CHECK(boolean(lobo_sqlite_close_v1(db)));
    else if(!strcmp(argv[1],"schema")) {execute(db,"CREATE TABLE users(id INTEGER, name TEXT)");execute(db,"CREATE INDEX users_name ON users(name)");execute(db,"DROP INDEX users_name");}
    else if(!strcmp(argv[1],"values")) values(db);
    else if(!strcmp(argv[1],"empty-invalid")) empty_and_invalid(db);
    else if(!strcmp(argv[1],"transactions")) transactions(db,argv[2]);
    else if(!strcmp(argv[1],"policy")) policy(db,cap);
    else if(!strcmp(argv[1],"injection")) injection(db);
    else if(!strcmp(argv[1],"budgets")) budgets(db);
    else if(!strcmp(argv[1],"stale")) stale(db);
    else if(!strcmp(argv[1],"busy")) busy(db,argv[2]);
    else if(!strcmp(argv[1],"commit-busy")) commit_busy(db,argv[2]);
    else if(!strcmp(argv[1],"memory")) memory_failure(db);
#if defined(__linux__)
    else if(!strcmp(argv[1],"symlink")) side_symlink(db,cap,argv[2]);
#endif
    else if(!strcmp(argv[1],"handles")) handle_quota(db);
    else if(!strcmp(argv[1],"generation")) generation(db);
    else if(is_fault) fault(db,argv[2],!strcmp(argv[1],"heap-cleanup"));
    else CHECK(false);
    puts("sqlite-adapter-case-passed");return 0;
}
