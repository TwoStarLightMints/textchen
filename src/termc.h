#ifndef TERMC
#define TERMC

struct wh;

unsigned int c_kbhit();
char get_ch();

#ifdef __linux__

// Required for Linux
void set_raw_term();
void set_cooked_term();

#endif

#ifdef _WIN32
#endif

// Required overall
struct wh get_term_size();

#endif
