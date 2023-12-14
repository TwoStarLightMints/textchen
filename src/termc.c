#include <stdio.h>
#include "termc.h"

#ifdef __linux__
#include <asm-generic/ioctls.h>
#include <termios.h>
#include <sys/ioctl.h>
#include <unistd.h>
#endif

#ifdef _WIN32
#include <conio.h>
#include <Windows.h>
#endif

struct wh {
  unsigned int width;
  unsigned int height;
};

// Set the terminal into raw mode (i.e. do not wait for the user to press return to accept and begin processing the input)
void set_raw_term() {
  #ifdef __linux__
  struct termios cooked;

  // Get the current settings for the terminal
  tcgetattr(0, &cooked);

  struct termios raw = cooked;

  // The c_lflag member of the termios struct handles terminal functions
  raw.c_lflag &= ~ECHO; // Turn off echo (when the user types something, don't show it on the screen)
  raw.c_lflag &= ~ICANON; // Turn off canonical mode (i.e. enter raw mode terminal)

  // Set the new setting for the terminal now
  tcsetattr(0, TCSANOW, &raw);

  #endif
}

// Set the terminal into cooked mode (i.e. return the terminal to its original state), simply do the reverse of the above
void set_cooked_term() {
  #ifdef __linux__
  struct termios raw;

  tcgetattr(0, &raw);

  struct termios cooked = raw;

  cooked.c_lflag &= ECHO;
  cooked.c_lflag &= ICANON;

  tcsetattr(0, TCSANOW, &cooked);;

  #endif
}

// Get a character from the user, easy enough to just implement in C
char get_ch() {
  char c;

  #ifdef __linux__
  scanf("%c", &c);
  #endif

  #ifdef _WIN32
  c = getch();
  #endif

  return c;
}

struct wh get_term_size() {
  #ifdef __linux__
  struct winsize w;
  
  ioctl(STDOUT_FILENO, TIOCGWINSZ, &w);

  struct wh res = { .width = w.ws_col, .height = w.ws_row };

  return res;

  #endif

  #ifdef _WIN32
  HANDLE hStdout = GetStdHandle(STD_OUTPUT_HANDLE);
  CONSOLE_SCREEN_BUFFER_INFO csbiInfo;

  if (!GetConsoleScreenBufferInfo(hStdout, &csbiInfo)) {
    fprintf(stderr, "Error getting console screen buffer info");
    exit(1);
  }

  struct wh res = { .width = (unsigned int)csbiInfo.dwSize.X, .height = (unsigned int)csbiInfo.dwSize.Y };

  return res;

  #endif
}
