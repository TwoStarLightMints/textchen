#ifndef TERMC
#define TERMC

struct wh;
void set_raw_term();
void set_cooked_term();
unsigned int c_kbhit();
struct wh get_term_size();
char get_ch();

#endif
