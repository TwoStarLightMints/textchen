#ifndef TERMC
#define TERMC

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

unsigned get_terminal_width();
unsigned get_terminal_width();

#endif
