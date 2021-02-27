#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

struct AppState;

struct OnMessage {
  void (*callback)(void*, const char *message);
  void *user_data;
};

extern "C" {

AppState *init_app();

void release_app(AppState *app_state);

void set_message_handler(AppState *app_state, const OnMessage *on_message);

bool run(AppState *app_state, const char *channel_name);

} // extern "C"
