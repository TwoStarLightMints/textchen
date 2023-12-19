#ifndef TERMC
#define TERMC

struct wh;

void set_raw_term();
void set_cooked_term();
char get_ch();
unsigned int l_kbhit();
struct wh get_term_size();

#endif
