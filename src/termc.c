#include <termios.h>
#include <stdio.h>
#include "termc.h"

// Set the terminal into raw mode (i.e. do not wait for the user to press return to accept and begin processing the input)
void set_raw_term() {
  struct termios cooked;

  // Get the current settings for the terminal
  tcgetattr(0, &cooked);

  struct termios raw = cooked;

  // The c_lflag member of the termios struct handles terminal functions
  raw.c_lflag &= ~ECHO; // Turn off echo (when the user types something, don't show it on the screen)
  raw.c_lflag &= ~ICANON; // Turn off canonical mode (i.e. enter raw mode terminal)

  // Set the new setting for the terminal now
  tcsetattr(0, TCSANOW, &raw);
}

// Set the terminal into cooked mode (i.e. return the terminal to its original state), simply do the reverse of the above
void set_cooked_term() {
  struct termios raw;

  tcgetattr(0, &raw);

  struct termios cooked = raw;

  cooked.c_lflag &= ECHO;
  cooked.c_lflag &= ICANON;

  tcsetattr(0, TCSANOW, &cooked);
}

// Get a character from the user, easy enough to just implement in C
char get_ch() {
  char c;

  scanf("%c", &c);

  return c;
}
