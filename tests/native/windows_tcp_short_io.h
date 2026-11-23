#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
static int lobo_test_send(SOCKET socket,const char *data,int count,int flags) {
    return send(socket,data,count>7?7:count,flags);
}
static int lobo_test_recv(SOCKET socket,char *data,int count,int flags) {
    return recv(socket,data,count>31?31:count,flags);
}
#define send lobo_test_send
#define recv lobo_test_recv
