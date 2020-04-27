#ifndef POD_LOG_H
#define POD_LOG_H

#include <stdio.h>
#include <stdbool.h>

// Source: https://github.com/oscarlab/graphene/blob/master/Pal/src/host/Linux-SGX/tools/common/util.h#L33
extern int g_stdout_fd;
extern int g_stderr_fd;
extern bool g_verbose;

#define DBG(fmt, ...)   do { if (g_verbose) dprintf(g_stdout_fd, fmt, ##__VA_ARGS__); } while (0)
#define INFO(fmt, ...)  do { dprintf(g_stdout_fd, fmt, ##__VA_ARGS__); } while (0)
#define ERROR(fmt, ...) do { dprintf(g_stderr_fd, "%s: " fmt, __FUNCTION__, ##__VA_ARGS__); } while (0)

void set_verbose(bool verbose);

#endif /* POD_LOG_H */
