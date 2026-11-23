#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
extern int lobo_generated_main(int,char **);
static char **utf8_arguments=NULL;
static int utf8_count=0;
static char database_path[4096];
static void free_arguments(void) {
    if(utf8_arguments) {for(int i=0;i<utf8_count;++i) free(utf8_arguments[i]);free(utf8_arguments);utf8_arguments=NULL;}
}
int wmain(int argc,wchar_t **wide) {
    utf8_arguments=calloc((size_t)argc+1,sizeof(char *));if(!utf8_arguments) return 1;
    utf8_count=argc;if(atexit(free_arguments)) {free_arguments();return 1;}
    for(int i=0;i<argc;++i) {
        int count=WideCharToMultiByte(CP_UTF8,WC_ERR_INVALID_CHARS,wide[i],-1,NULL,0,NULL,NULL);
        if(count<=0 || !(utf8_arguments[i]=malloc((size_t)count)) || !WideCharToMultiByte(CP_UTF8,WC_ERR_INVALID_CHARS,wide[i],-1,utf8_arguments[i],count,NULL,NULL)) return 1;
    }
    char **argv=utf8_arguments;
    if(argc!=1) return lobo_generated_main(argc,argv);
    wchar_t path[1024];DWORD length=GetFullPathNameW(L"lobo-users.sqlite",1024,path,NULL);
    if(!length || length>=1024 || !WideCharToMultiByte(CP_UTF8,WC_ERR_INVALID_CHARS,path,-1,database_path,sizeof(database_path),NULL,NULL)) return 1;
    char *defaults[]={argv[0],"--entry","example.sqlite.rest.main","--arg","I64:8080","--arg","I64:128","--arg","Bool:true",
        "--native-clock","true","--native-listen","127.0.0.1:8080","--native-calls","65536","--native-bytes","1048576",
        "--sqlite-file",database_path,"--sqlite-mode","schema","--sqlite-calls","65536","--sqlite-steps","1000000",
        "--fuel","100000000","--heap-cells","16000000",NULL};
    puts("Starting Lobo SQLite REST API at http://127.0.0.1:8080/health");
    puts("Users persist in lobo-users.sqlite. Try /api/users and /api/users/1.");fflush(stdout);
    int code=lobo_generated_main((int)(sizeof(defaults)/sizeof(*defaults))-1,defaults);
    DWORD mode;
    if(GetConsoleMode(GetStdHandle(STD_INPUT_HANDLE),&mode)) {puts("Press Enter to close.");(void)getchar();}
    return code;
}
