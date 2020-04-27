#include "pod_log.h"

int g_stdout_fd = 1;
int g_stderr_fd = 2;
bool g_verbose = false;

void set_verbose(bool verbose) {
  g_verbose = verbose;
  if (verbose)
    DBG("Verbose output enabled\n");
  else
    DBG("Verbose output disabled\n");
}
