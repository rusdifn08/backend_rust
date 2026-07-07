import os
import re

# 1. Update main.rs
main_path = 'src/main.rs'
with open(main_path, 'r') as f:
    content = f.read()

content = content.replace(
    '.route(\"/api/todos/:id\", put(api::todos::toggle_todo).delete(api::todos::delete_todo))',
    '.route(\"/api/todos/:id\", put(api::todos::update_todo).patch(api::todos::toggle_todo).delete(api::todos::delete_todo))'
)
content = content.replace(
    '.route(\"/api/habits/:id\", put(api::habits::toggle_habit).delete(api::habits::delete_habit))',
    '.route(\"/api/habits/:id\", put(api::habits::update_habit).patch(api::habits::toggle_habit).delete(api::habits::delete_habit))'
)
content = content.replace(
    '.route(\"/api/notes/:id\", delete(api::notes::delete_note))',
    '.route(\"/api/notes/:id\", put(api::notes::update_note).delete(api::notes::delete_note))'
)
with open(main_path, 'w') as f:
    f.write(content)

# 2. Add UpdateNoteReq
note_model = 'src/models/note.rs'
with open(note_model, 'a') as f:
    f.write('\n#[derive(Debug, serde::Deserialize)]\npub struct UpdateNoteReq {\n    pub title: String,\n    pub content: String,\n    pub date: String,\n    pub tag: String,\n    pub color: String,\n    pub deadline: Option<String>,\n}\n')

# 3. Add UpdateHabitReq
habit_model = 'src/models/habit.rs'
with open(habit_model, 'a') as f:
    f.write('\n#[derive(Debug, serde::Deserialize)]\npub struct UpdateHabitReq {\n    pub title: String,\n    pub subtitle: String,\n    pub category: String,\n    pub target_days: i32,\n    pub color: String,\n    pub icon: Option<String>,\n}\n')

# 4. Add update_note to notes.rs
notes_api = 'src/api/notes.rs'
with open(notes_api, 'r') as f:
    notes_content = f.read()
notes_content = notes_content.replace('use crate::models::note::{Note, CreateNoteReq};', 'use crate::models::note::{Note, CreateNoteReq, UpdateNoteReq};')
update_note_func = '''
pub async fn update_note(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateNoteReq>,
) -> Result<Json<Note>, (StatusCode, String)> {
    let note = sqlx::query_as::<_, Note>(
        r#"
        UPDATE notes 
        SET title = , content = , date = , tag = , color = , deadline = , updated_at = NOW() 
        WHERE id =  
        RETURNING *
        "#
    )
    .bind(req.title)
    .bind(req.content)
    .bind(req.date)
    .bind(req.tag)
    .bind(req.color)
    .bind(req.deadline)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(note))
}
'''
if 'pub async fn update_note' not in notes_content:
    notes_content += update_note_func
with open(notes_api, 'w') as f:
    f.write(notes_content)

# 5. Add update_habit to habits.rs
habits_api = 'src/api/habits.rs'
with open(habits_api, 'r') as f:
    habits_content = f.read()
habits_content = habits_content.replace('use crate::models::habit::{Habit, CreateHabitReq, HabitLog, CreateHabitLogReq};', 'use crate::models::habit::{Habit, CreateHabitReq, UpdateHabitReq, HabitLog, CreateHabitLogReq};')
update_habit_func = '''
pub async fn update_habit(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateHabitReq>,
) -> Result<Json<Habit>, (StatusCode, String)> {
    let habit = sqlx::query_as::<_, Habit>(
        r#"
        UPDATE habits 
        SET title = , subtitle = , category = , target_days = , color = , icon = , updated_at = NOW() 
        WHERE id =  
        RETURNING *
        "#
    )
    .bind(req.title)
    .bind(req.subtitle)
    .bind(req.category)
    .bind(req.target_days)
    .bind(req.color)
    .bind(req.icon)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(habit))
}
'''
if 'pub async fn update_habit' not in habits_content:
    habits_content += update_habit_func
with open(habits_api, 'w') as f:
    f.write(habits_content)

print('Backend updated successfully!')
