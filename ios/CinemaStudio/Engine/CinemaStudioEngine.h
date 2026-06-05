#ifndef CinemaStudioEngine_h
#define CinemaStudioEngine_h

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

void cs_c_free_string(char *s);
int32_t cs_c_engine_init(const char *data_root);
char *cs_c_bridge_status(void);
char *cs_c_create_project(const char *name, const char *parent_dir);
char *cs_c_open_project(const char *project_dir);
int32_t cs_c_save_project(void);
char *cs_c_import_media(const char *source_path);
char *cs_c_scrub_to(uint64_t time_ms);
int32_t cs_c_playback_play(void);
int32_t cs_c_playback_pause(void);
char *cs_c_playback_tick(void);
int32_t cs_c_undo(void);
int32_t cs_c_redo(void);
int32_t cs_c_can_undo(void);
int32_t cs_c_can_redo(void);
char *cs_c_ai_suggestions(void);
int32_t cs_c_set_decoder_backend(const char *name);
int32_t cs_c_set_render_backend(const char *name);

#ifdef __cplusplus
}
#endif

#endif /* CinemaStudioEngine_h */
