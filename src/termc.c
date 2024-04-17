#include "termc.h"

#include <stdio.h>

struct wh {
  unsigned int width;
  unsigned int height;
};

#ifdef __linux__

#include <termios.h>
#include <sys/ioctl.h>
#include <unistd.h>
#include <signal.h>

struct wh get_term_size() {
  struct winsize w;

  ioctl(STDOUT_FILENO, TIOCGWINSZ, &w);

  struct wh res = { .width = w.ws_col, .height = w.ws_row };

  return res;

}

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

  cooked.c_lflag |= ECHO;
  cooked.c_lflag |= ICANON;

  tcsetattr(0, TCSANOW, &cooked);;

}

// Get a character from the user, easy enough to just implement in C
char get_ch() {
  char c;

  scanf("%c", &c);

  return c;
}

// Check if keyboard key was hit
unsigned int c_kbhit() {
  int waiting;

  ioctl(STDIN_FILENO, FIONREAD, &waiting);

  return waiting > 0;
}

#endif

#ifdef _WIN32

#include <Windows.h>
#include <conio.h>

struct wh get_term_size() {
  HANDLE hConsoleOutput;
  CONSOLE_SCREEN_BUFFER_INFO csbi;

  hConsoleOutput = GetStdHandle(STD_OUTPUT_HANDLE);

  GetConsoleScreenBufferInfo(hConsoleOutput, &csbi);

  struct wh widthHeight = {
    .width = csbi.dwSize.X,
    .height = csbi.dwSize.Y
    };

    return widthHeight;
}

char get_ch() {
  return (char) _getch();
}

unsigned int c_kbhit() {
  return _kbhit();
}

#endif
