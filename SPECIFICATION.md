# Hugo Blog Editor - Техническая Спецификация

## Обзор Проекта

**Название:** Hugo Blog Editor (Hugo Bros)
**Тип:** Desktop GUI приложение для редактирования Hugo блогов
**Целевая аудитория:** Пользователи Hugo, желающие иметь визуальный редактор вместо работы с markdown файлами напрямую
**Платформы:** macOS (Intel + Apple Silicon), Linux, Windows 10/11

## Технический Стек

### Backend (Rust)
- **Framework:** Tauri 2.x
- **Парсинг Markdown:** `pulldown-cmark` или `comrak`
- **Парсинг YAML frontmatter:** `serde_yaml`
- **Работа с файлами:** `std::fs`, `walkdir`
- **Выполнение команд:** `std::process::Command`
- **Конфигурация:** `serde` + `toml` / `serde_json`

### Frontend (TypeScript + Svelte)
- **Framework:** Svelte 4.x + SvelteKit
- **WYSIWYG Editor:** TipTap 2.x (ProseMirror-based)
- **Markdown парсинг (frontend):** `marked` или TipTap Markdown extension
- **Подсветка синтаксиса:** `highlight.js` или `prism.js`
- **UI Components:** Custom компоненты + shadcn-svelte (опционально)
- **Стилизация:** TailwindCSS
- **Иконки:** Lucide Svelte
- **Локализация:** `svelte-i18n`

### Дополнительные библиотеки
- **Drag & Drop:** Native HTML5 Drag & Drop API
- **Превью изображений:** Native HTML `<img>`
- **Файловый диалог:** Tauri API (`dialog`)
- **Системная тема:** Tauri API (`theme`)

## Архитектура Приложения

### Структура проекта

```
hugo-bros/
├── src-tauri/           # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── commands.rs  # Tauri commands
│   │   ├── hugo.rs      # Hugo integration
│   │   ├── config.rs    # App configuration
│   │   ├── markdown.rs  # Markdown parsing
│   │   └── files.rs     # File operations
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                 # Svelte frontend
│   ├── lib/
│   │   ├── components/  # UI components
│   │   ├── stores/      # Svelte stores (state)
│   │   ├── i18n/        # Локализация
│   │   └── utils/       # Helpers
│   ├── routes/          # SvelteKit routes
│   └── app.html
│
├── static/              # Static assets
├── package.json
└── svelte.config.js
```

### Backend Commands (Tauri)

```rust
// Основные команды, вызываемые из frontend

// Проект
#[tauri::command]
fn select_project_folder() -> Result<String, String>

#[tauri::command]
fn get_project_config(project_path: String) -> Result<HugoConfig, String>

// Посты и страницы
#[tauri::command]
fn list_posts(project_path: String) -> Result<Vec<Post>, String>

#[tauri::command]
fn list_pages(project_path: String) -> Result<Vec<Page>, String>

#[tauri::command]
fn list_drafts(project_path: String) -> Result<Vec<Draft>, String>

#[tauri::command]
fn get_post(project_path: String, post_id: String) -> Result<Post, String>

#[tauri::command]
fn save_post(project_path: String, post: Post) -> Result<(), String>

#[tauri::command]
fn create_post(project_path: String, title: String) -> Result<Post, String>

#[tauri::command]
fn delete_post(project_path: String, post_id: String) -> Result<(), String>

// Изображения
#[tauri::command]
fn list_images(project_path: String) -> Result<Vec<ImageInfo>, String>

#[tauri::command]
fn upload_image(project_path: String, image_data: Vec<u8>, filename: String) -> Result<String, String>

#[tauri::command]
fn copy_image_to_project(project_path: String, source_path: String) -> Result<String, String>

// Hugo команды
#[tauri::command]
fn run_hugo_command(project_path: String, command: String) -> Result<CommandOutput, String>

#[tauri::command]
fn start_hugo_server(project_path: String) -> Result<String, String>

#[tauri::command]
fn stop_hugo_server(server_id: String) -> Result<(), String>

// Конфигурация
#[tauri::command]
fn get_app_config() -> Result<AppConfig, String>

#[tauri::command]
fn save_app_config(config: AppConfig) -> Result<(), String>
```

### Data Models

```rust
// Rust structures

#[derive(Serialize, Deserialize)]
struct Post {
    id: String,              // Путь к файлу относительно content/posts/
    title: String,
    date: String,            // ISO 8601 datetime
    content: String,         // Markdown content (без frontmatter)
    frontmatter: Frontmatter,
    file_path: String,       // Полный путь к файлу
    created_at: i64,         // Unix timestamp
    modified_at: i64,        // Unix timestamp
}

#[derive(Serialize, Deserialize)]
struct Frontmatter {
    title: String,
    date: String,
    tags: Vec<String>,
    categories: Vec<String>,
    permalink: Option<String>,
    list_image: Option<String>,
    list_image_alt: Option<String>,
    main_image: Option<String>,
    main_image_alt: Option<String>,
    // Дополнительные поля как HashMap для гибкости
    custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct Page {
    id: String,
    title: String,
    content: String,
    frontmatter: Frontmatter,
    file_path: String,
    created_at: i64,
    modified_at: i64,
}

#[derive(Serialize, Deserialize)]
struct Draft {
    id: String,
    title: String,
    content: String,
    frontmatter: Frontmatter,
    file_path: String,
    created_at: i64,
    modified_at: i64,
}

#[derive(Serialize, Deserialize)]
struct ImageInfo {
    filename: String,
    path: String,           // Относительный путь от static/images/
    full_path: String,      // Полный системный путь
    url: String,            // URL для вставки в markdown (/images/...)
    size: u64,              // Размер в байтах
    width: Option<u32>,
    height: Option<u32>,
    created_at: i64,
}

#[derive(Serialize, Deserialize)]
struct HugoConfig {
    title: String,
    subtitle: String,
    description: String,
    author: String,
    language: String,
    url: String,
    // Другие поля из config.*
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    version: String,
    last_project_path: Option<String>,
    recent_projects: Vec<String>,
    ui_language: String,      // "ru" | "en"
    theme: String,            // "light" | "dark" | "auto"
    auto_save_enabled: bool,
    auto_save_interval: u32,  // Секунды
    editor_font_size: u32,
    editor_line_height: f32,
}

#[derive(Serialize, Deserialize)]
struct CommandOutput {
    success: bool,
    stdout: String,
    stderr: String,
    exit_code: i32,
}
```

## Функциональные Требования

### 1. MVP (Минимально Жизнеспособный Продукт)

#### 1.1. Базовый Markdown Редактор (Приоритет 1)

**Описание:** WYSIWYG редактор на основе TipTap с возможностью редактирования markdown и просмотра результата.

**Функциональность:**
- WYSIWYG режим для редактирования (TipTap)
- Markdown синтаксис:
  - Заголовки (H1-H6)
  - Жирный, курсив, зачеркнутый текст
  - Списки (упорядоченные, неупорядоченные)
  - Цитаты (blockquotes)
  - Ссылки
  - Изображения
  - Блоки кода с подсветкой синтаксиса
  - Горизонтальные линии
  - Таблицы (опционально для MVP)
- Горячие клавиши:
  - `Ctrl/Cmd + B` - жирный
  - `Ctrl/Cmd + I` - курсив
  - `Ctrl/Cmd + K` - вставка ссылки
  - `Ctrl/Cmd + Shift + C` - блок кода
  - `Ctrl/Cmd + S` - сохранить
  - `Ctrl/Cmd + Z/Y` - отмена/повтор
- Панель инструментов (toolbar) с кнопками форматирования
- Счетчик слов и символов
- Индикатор сохранения (saved / saving / unsaved)

**UI Layout:**
```
┌─────────────────────────────────────────────────────┐
│ [Назад] Post Title                    [Сохранить]   │
├─────────────────────────────────────────────────────┤
│ [B] [I] [U] [H1▾] [Link] [Image] [Code] [List▾]    │ ← Toolbar
├─────────────────────────────────────────────────────┤
│                                                     │
│                                                     │
│           WYSIWYG Editor Area (TipTap)              │
│                                                     │
│                                                     │
│                                                     │
├─────────────────────────────────────────────────────┤
│ 245 слов • 1523 символа              ● Сохранено   │
└─────────────────────────────────────────────────────┘
```

#### 1.2. Редактирование Frontmatter (Приоритет 2)

**Описание:** Визуальный редактор для YAML frontmatter с поддержкой всех полей.

**Функциональность:**
- Поля для редактирования:
  - Title (текстовое поле)
  - Date (date picker)
  - Tags (multi-select с возможностью добавления новых)
  - Categories (multi-select)
  - Permalink (текстовое поле)
  - List Image (выбор из галереи + drag-and-drop)
  - List Image Alt (текстовое поле)
  - Main Image (выбор из галереи + drag-and-drop)
  - Main Image Alt (текстовое поле)
  - Произвольные поля (key-value пары)
- Валидация полей:
  - Title - обязательное
  - Date - формат ISO 8601
  - Images - проверка существования файла
- Превью изображений в полях

**UI Layout:**
```
┌─────────────────────────────────────────────────────┐
│ Frontmatter                                [Свернуть]│
├─────────────────────────────────────────────────────┤
│ Title: [Отчет: 2024________________________]        │
│ Date:  [31.12.2024] [12:00]                         │
│ Tags:  [×ithink] [+Добавить тег]                    │
│ Permalink: [2024report___________________]          │
│                                                     │
│ List Image:  [──────────] или перетащите            │
│              ┌──────┐                               │
│              │ img  │  ny_2025_thumb.jpg            │
│              └──────┘                               │
│ Alt: [С Новым Годом______________________]          │
│                                                     │
│ Main Image:  [──────────] или перетащите            │
│              ┌────────┐                             │
│              │  img   │  ny_2025.jpg                │
│              └────────┘                             │
│ Alt: [С Новым Годом______________________]          │
│                                                     │
│ [+ Добавить поле]                                   │
└─────────────────────────────────────────────────────┘
```

#### 1.3. Работа с Изображениями (Приоритет 3)

**Описание:** Файловый менеджер для управления изображениями блога.

**Функциональность:**
- Просмотр всех изображений из `static/images/`
- Превью изображений (thumbnail grid)
- Информация об изображении (имя, размер, разрешение, дата)
- Загрузка изображений:
  - Drag-and-drop в зону загрузки
  - Выбор через диалог
  - Вставка из буфера обмена (Ctrl+V)
- Автоматическое копирование изображений в `static/images/`
- Генерация уникальных имен при конфликтах
- Вставка изображения в редактор (клик по изображению)
- Вставка изображения в frontmatter (drag-and-drop на поле)
- Удаление изображений (с подтверждением)
- Поиск по имени изображения
- Сортировка (по дате, имени, размеру)

**UI Layout (модальное окно):**
```
┌─────────────────────────────────────────────────────┐
│ Галерея изображений                    [Загрузить] │
├─────────────────────────────────────────────────────┤
│ [Поиск изображений____________________] [Сортировка▾]│
├─────────────────────────────────────────────────────┤
│ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐            │
│ │ img │ │ img │ │ img │ │ img │ │ img │            │
│ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘            │
│ name1   name2   name3   name4   name5              │
│                                                     │
│ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐            │
│ │ img │ │ img │ │ img │ │ img │ │ img │            │
│ └─────┘ └─────┘ └─────┘ └─────┘ └─────┘            │
│ name6   name7   name8   name9   name10             │
├─────────────────────────────────────────────────────┤
│ Выбрано: screenshot.png (245KB, 1920x1080)          │
│                             [Вставить] [Отмена]     │
└─────────────────────────────────────────────────────┘
```

#### 1.4. Список Постов с Поиском (Приоритет 4)

**Описание:** Главный экран со списком всех постов, страниц и черновиков.

**Функциональность:**
- Отображение постов в виде карточек:
  - Превью изображения (list_image из frontmatter)
  - Заголовок
  - Дата публикации
  - Первые 150 символов текста
  - Теги
- Фильтрация:
  - По типу (посты / страницы / черновики)
  - По тегам
  - По дате
- Поиск:
  - По заголовку
  - По содержимому
  - По тегам
- Сортировка:
  - По дате (новые первыми / старые первыми)
  - По названию (A-Z / Z-A)
  - По дате изменения
- Действия с постом:
  - Открыть для редактирования
  - Удалить (с подтверждением)
  - Преобразовать черновик в пост
  - Дублировать
- Создание нового поста (кнопка "+" / FAB)

**UI Layout:**
```
┌─────────────────────────────────────────────────────┐
│ Hugo Editor                              [⚙] [👤]   │
├─────────────────────────────────────────────────────┤
│ [Посты] [Страницы] [Черновики]           [+ Новый]  │
├─────────────────────────────────────────────────────┤
│ [Поиск постов____________________________] [Фильтр▾]│
├─────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────┐ │
│ │ ┌──────────┐                                    │ │
│ │ │          │  Отчет: 2024                       │ │
│ │ │  Image   │  31 декабря 2024                   │ │
│ │ │          │  Всем привет! Если честно в этот...│ │
│ │ └──────────┘  #ithink                           │ │
│ └─────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────┐ │
│ │ ┌──────────┐                                    │ │
│ │ │          │  Weekly Digest                     │ │
│ │ │  Image   │  25 декабря 2024                   │ │
│ │ │          │  Подборка интересных ссылок за...  │ │
│ │ └──────────┘  #digest #links                    │ │
│ └─────────────────────────────────────────────────┘ │
│ ┌─────────────────────────────────────────────────┐ │
│ │ ┌──────────┐                                    │ │
│ │ │   [?]    │  Docker и OpenVPN                  │ │
│ │ │  No img  │  20 ноября 2024                    │ │
│ │ │          │  Настройка OpenVPN сервера через...│ │
│ │ └──────────┘  #docker #vpn                      │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

### 2. Дополнительные Функции (Post-MVP)

#### 2.1. Hugo Интеграция

**Описание:** Запуск Hugo команд из GUI.

**Функциональность:**
- Запуск локального сервера (`hugo server`)
  - Индикатор статуса (запущен/остановлен)
  - Отображение URL (http://localhost:1313)
  - Кнопка "Открыть в браузере"
  - Кнопка "Остановить сервер"
- Сборка статики (`hugo`)
  - Progress bar
  - Логи выполнения
  - Уведомление об успехе/ошибке
- Деплой (`hugo deploy`)
  - Подтверждение перед деплоем
  - Progress bar
  - Логи выполнения
- Очистка (`hugo --gc --cleanDestinationDir`)
- Вывод логов команд в отдельной панели

#### 2.2. Выбор Проекта

**Описание:** Выбор папки с Hugo проектом при запуске приложения.

**Функциональность:**
- При первом запуске - диалог выбора папки
- Валидация выбранной папки:
  - Проверка наличия `config.*` или `hugo.*`
  - Проверка наличия `content/` папки
  - Проверка наличия `package.json` с hugo
- Список недавних проектов (до 10)
- Возможность переключения между проектами
- Сохранение последнего открытого проекта

#### 2.3. Автосохранение

**Описание:** Автоматическое и ручное сохранение изменений.

**Функциональность:**
- Автосохранение каждые N секунд (настраиваемо, по умолчанию 30 сек)
- Индикатор статуса сохранения:
  - "Сохранено" (зеленый)
  - "Сохранение..." (желтый)
  - "Не сохранено" (красный)
- Ручное сохранение по Ctrl/Cmd+S
- Предупреждение при закрытии несохраненного поста
- История изменений (опционально)

#### 2.4. Настройки Приложения

**Описание:** Страница настроек приложения.

**Функциональность:**
- Язык интерфейса (русский / английский)
- Тема (светлая / темная / авто)
- Автосохранение (включено / отключено)
- Интервал автосохранения (секунды)
- Размер шрифта редактора
- Высота строки редактора
- Последние проекты (список с возможностью очистки)

#### 2.5. Создание Постов/Страниц

**Описание:** Диалоги для создания новых постов, страниц и черновиков.

**Функциональность:**
- Создание поста:
  - Диалог с полем "Название"
  - Автоматическая генерация permalink из названия (транслитерация)
  - Создание файла в `content/posts/`
  - Заполнение frontmatter шаблоном
  - Открытие в редакторе
- Создание страницы:
  - Диалог с полем "Название"
  - Создание папки в `content/`
  - Создание `index.md`
- Создание черновика:
  - Аналогично посту, но в `content/drafts/`
- Преобразование черновика в пост:
  - Перемещение файла из `_drafts` в `_posts`
  - Обновление даты

### 3. UI/UX Требования

#### 3.1. Дизайн Система

**Стиль:** Минималистичный (как Notion, Bear)

**Принципы:**
- Чистота и простота
- Много белого/серого пространства
- Фокус на контенте
- Минимум отвлекающих элементов
- Мягкие тени и скругленные углы (4-8px border-radius)

**Цветовая Палитра:**

Светлая тема:
- Background: `#FFFFFF`
- Surface: `#F7F7F7`
- Border: `#E5E5E5`
- Text Primary: `#1A1A1A`
- Text Secondary: `#666666`
- Accent: `#3B82F6` (Blue)
- Success: `#10B981` (Green)
- Warning: `#F59E0B` (Amber)
- Error: `#EF4444` (Red)

Темная тема:
- Background: `#1A1A1A`
- Surface: `#2D2D2D`
- Border: `#404040`
- Text Primary: `#F5F5F5`
- Text Secondary: `#A3A3A3`
- Accent: `#60A5FA` (Blue)
- Success: `#34D399` (Green)
- Warning: `#FBBF24` (Amber)
- Error: `#F87171` (Red)

**Типографика:**
- Заголовки: Inter / SF Pro Display (система)
- Основной текст: Inter / SF Pro Text (система)
- Моноширинный (код): JetBrains Mono / SF Mono (система)
- Размеры:
  - H1: 32px
  - H2: 24px
  - H3: 20px
  - Body: 16px
  - Small: 14px
  - Caption: 12px

**Spacing:**
- Base unit: 4px
- Common spacing: 8px, 12px, 16px, 24px, 32px, 48px

#### 3.2. Компоненты

Переиспользуемые UI компоненты:
- Button (primary, secondary, ghost, danger)
- Input (text, number, date, search)
- Textarea
- Select / Dropdown
- Multi-select (tags)
- Checkbox
- Radio
- Toggle/Switch
- Modal/Dialog
- Tooltip
- Card
- Badge
- Dropdown Menu
- Date Picker
- File Upload Zone

#### 3.3. Навигация

**Главное Меню:**
- Список постов (главная)
- Настройки
- О программе

**Toolbar Редактора:**
- Кнопка "Назад" (к списку постов)
- Заголовок текущего поста
- Кнопка "Сохранить"
- Кнопка "Настройки поста" (frontmatter toggle)

#### 3.4. Адаптивность

- Минимальный размер окна: 1024x768
- Рекомендуемый размер окна: 1280x800
- Поддержка различных DPI (HiDPI / Retina)
- Масштабирование UI через настройки

#### 3.5. Анимации

- Переходы между экранами: 200-300ms ease-in-out
- Hover эффекты: 150ms
- Модальные окна: fade + scale 200ms
- Использовать CSS transitions, избегать тяжелых JS анимаций

### 4. Нефункциональные Требования

#### 4.1. Производительность

- Запуск приложения: < 2 секунды
- Открытие поста: < 500ms
- Сохранение поста: < 200ms
- Загрузка списка постов (100 постов): < 1 секунда
- Поиск по постам: < 300ms (с debounce 300ms)
- Загрузка изображения: < 500ms

#### 4.2. Надежность

- Автосохранение для предотвращения потери данных
- Обработка ошибок:
  - Ошибки чтения/записи файлов
  - Ошибки парсинга YAML/Markdown
  - Ошибки выполнения Hugo команд
- Логирование ошибок (в файл для отладки)
- Graceful degradation при отсутствии интернета

#### 4.3. Безопасность

- Валидация путей к файлам (предотвращение path traversal)
- Санитизация пользовательского ввода
- Безопасное выполнение shell команд
- Не хранить sensitive данные в plaintext (если будут)

#### 4.4. Локализация

- Поддержка русского и английского языков
- i18n файлы в формате JSON
- Переключение языка в настройках (требуется перезапуск или live reload)
- Форматирование дат согласно локали

#### 4.5. Кроссплатформенность

- Единая кодовая база для всех платформ
- Использование системных диалогов через Tauri API
- Учет особенностей платформ:
  - macOS: Cmd вместо Ctrl, системное меню
  - Windows: Window controls, нативные диалоги
  - Linux: GTK file chooser, системная тема

#### 4.6. Обновления

- Проверка обновлений при запуске (опционально)
- Автоматическая загрузка и установка обновлений (через Tauri updater)
- Changelog в модальном окне

## План Разработки (Фазы)

### Фаза 0: Подготовка (1-2 дня)
- [ ] Создание нового Tauri + Svelte проекта
- [ ] Настройка TailwindCSS
- [ ] Настройка TypeScript
- [ ] Настройка линтеров (ESLint, Prettier)
- [ ] Базовая структура проекта
- [ ] Подключение необходимых зависимостей

### Фаза 1: MVP - Базовая Функциональность (2-3 недели)

#### Week 1: Backend + File Operations
- [ ] Реализация Rust команд для работы с файлами
- [ ] Парсинг Markdown и YAML frontmatter
- [ ] Чтение/запись постов и страниц
- [ ] Работа с изображениями (копирование, список)
- [ ] Базовая структура AppConfig

#### Week 2: Editor + UI
- [ ] Интеграция TipTap редактора
- [ ] Toolbar с базовыми кнопками форматирования
- [ ] Горячие клавиши
- [ ] Подсветка синтаксиса в блоках кода
- [ ] Счетчик слов
- [ ] UI компоненты (Button, Input, Modal)
- [ ] Редактор Frontmatter

#### Week 3: Post List + Image Gallery
- [ ] Список постов с карточками
- [ ] Поиск и фильтрация
- [ ] Создание/удаление постов
- [ ] Галерея изображений (modal)
- [ ] Drag-and-drop изображений
- [ ] Вставка изображений в редактор и frontmatter

### Фаза 2: Полировка и Дополнительные Функции (1-2 недели)

#### Week 4: Hugo Integration + Project Selection
- [ ] Выбор папки проекта
- [ ] Валидация Hugo проекта
- [ ] Запуск Hugo команд (server, generate, deploy)
- [ ] Индикатор статуса сервера
- [ ] Панель логов

#### Week 5: Settings + Localization + Themes
- [ ] Страница настроек
- [ ] Переключение темы (светлая/темная/авто)
- [ ] i18n (русский + английский)
- [ ] Автосохранение
- [ ] История проектов

### Фаза 3: Тестирование и Релиз (1 неделя)

#### Week 6: Testing + Documentation
- [ ] Тестирование на всех платформах (macOS, Linux, Windows)
- [ ] Исправление багов
- [ ] Написание README
- [ ] Создание скриншотов и демо-видео
- [ ] Подготовка релиза
- [ ] Публикация на GitHub

## Риски и Ограничения

### Технические Риски

1. **TipTap + Markdown конверсия**
   - **Риск:** Потеря форматирования при конвертации между Markdown и TipTap JSON
   - **Митигация:** Использовать TipTap Markdown extension, тестировать edge cases

2. **Парсинг YAML frontmatter**
   - **Риск:** Некорректный парсинг сложных YAML структур
   - **Митигация:** Использовать проверенные библиотеки (serde_yaml), обработка ошибок

3. **Производительность при большом количестве постов**
   - **Риск:** Медленная загрузка списка при 1000+ постов
   - **Митигация:** Виртуализация списка, пагинация, индексация

4. **Кроссплатформенные различия**
   - **Риск:** Разное поведение на разных ОС
   - **Митигация:** Тестирование на всех платформах, использование Tauri API

### Функциональные Ограничения

1. **Нет работы с темами Hugo**
   - Редактор не управляет темами, только контентом

2. **Нет поддержки плагинов Hugo**
   - Только базовая функциональность Hugo

3. **Нет встроенного Git**
   - Пользователь должен использовать внешние Git клиенты

4. **Нет коллаборации**
   - Только локальное редактирование, без real-time collaboration

## Метрики Успеха

- Время создания нового поста: < 1 минуты
- Время редактирования существующего поста: старт через 2-3 секунды
- Вставка изображения: < 10 секунд (включая drag-and-drop)
- Экономия времени vs ручное редактирование файлов: 50%
- Поддержка 99% Hugo markdown синтаксиса
- Запуск на всех 3 платформах без критических багов
- Размер приложения: < 50MB

## Дальнейшее Развитие

### Post-MVP Фичи

1. **Улучшенный редактор**
   - Split view (markdown code + preview)
   - Vim mode
   - Distraction-free mode
   - Markdown shortcuts (type `##` для заголовка)

2. **Расширенная работа с контентом**
   - Категории (sidebar навигация)
   - Экспорт в PDF/HTML
   - Импорт из других платформ (Medium, WordPress)
   - Статистика блога (количество постов, слов, просмотров)

3. **Интеграция с сервисами**
   - Git интеграция (commit, push)
   - Публикация на Netlify/Vercel одним кликом
   - Интеграция с image CDN (Cloudinary, ImgIX)
   - SEO оптимизация (meta tags, Open Graph)

4. **Коллаборация**
   - Комментарии к постам (для команды)
   - История изменений (Git-подобная)
   - Conflict resolution

5. **Расширенные настройки**
   - Пользовательские поля frontmatter (конфигурируемые)
   - Шаблоны постов (templates)
   - Snippets
   - Keyboard shortcuts customization

6. **Мобильное приложение**
   - iOS/Android версии (с синхронизацией)

## Заключение

Данная спецификация описывает полнофункциональный GUI редактор для Hugo блогов на базе Tauri + Svelte. Проект фокусируется на минималистичном UX, производительности и кроссплатформенности.

**Ключевые особенности:**
- WYSIWYG Markdown редактор (TipTap)
- Визуальный редактор frontmatter
- Файловый менеджер изображений с drag-and-drop
- Список постов с карточками и поиском
- Hugo интеграция (server, generate, deploy)
- Светлая/темная тема с автопереключением
- Локализация (русский + английский)
- Автосохранение

**MVP можно реализовать за 4-6 недель** одним разработчиком при условии знания Rust + Svelte + Tauri.

Следующий шаг: начать с Фазы 0 (подготовка проекта) и постепенно двигаться по плану разработки.
