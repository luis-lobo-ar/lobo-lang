#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdio.h>
extern int lobo_generated_main(int,char **);
int main(int argc,char **argv) {
    if(argc!=1) return lobo_generated_main(argc,argv);
    char *defaults[]={argv[0],"--entry","example.rest.main","--arg","I64:8080","--arg","I64:128",
        "--native-clock","true","--native-listen","127.0.0.1:8080","--native-calls","65536",
        "--native-bytes","1048576","--fuel","100000000","--heap-cells","16000000",NULL};
    puts("Starting Lobo REST API at http://127.0.0.1:8080/health");
    puts("Try /api/users and /api/users/1. Press Ctrl+C to stop.");fflush(stdout);
    int code=lobo_generated_main((int)(sizeof(defaults)/sizeof(defaults[0]))-1,defaults);
    DWORD mode;
    if(GetConsoleMode(GetStdHandle(STD_INPUT_HANDLE),&mode)) {puts("Press Enter to close.");(void)getchar();}
    return code;
}
