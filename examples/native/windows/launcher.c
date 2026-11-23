/* The app body is generated from snapshot.lobo; this only supplies demo launch policy and console UX. */
#include <stdio.h>
#include <string.h>
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
int lobo_generated_main(int argc,char **argv);
int main(int argc,char **argv) {
    int no_pause=argc==2 && !strcmp(argv[1],"--no-pause");
    if(argc!=1 && !no_pause) return lobo_generated_main(argc,argv);
    char *demo[]={argv[0],"--entry","native.snapshot.main","--native-clock","true","--native-random","true"};
    int result=lobo_generated_main(7,demo);
    DWORD mode=0;
    if(!no_pause && GetConsoleMode(GetStdHandle(STD_INPUT_HANDLE),&mode)) {
        fputs("\nPress Enter to close...",stdout);fflush(stdout);(void)getchar();
    }
    return result;
}
