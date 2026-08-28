CREATE TABLE clipboard_items (
    id TEXT PRIMARY KEY NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('text', 'image', 'file', 'url', 'code')),
    content TEXT NOT NULL,
    normalized_content TEXT NOT NULL,
    preview TEXT NOT NULL,
    pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1)),
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_clipboard_items_created_at ON clipboard_items (created_at DESC);
CREATE INDEX idx_clipboard_items_type_created_at ON clipboard_items (type, created_at DESC);
CREATE INDEX idx_clipboard_items_pinned_created_at ON clipboard_items (pinned, created_at DESC);
CREATE INDEX idx_clipboard_items_normalized ON clipboard_items (normalized_content);

CREATE VIRTUAL TABLE clipboard_items_fts USING fts5(
    content,
    normalized_content,
    content = 'clipboard_items',
    content_rowid = 'rowid',
    tokenize = 'unicode61'
);

CREATE TRIGGER clipboard_items_ai AFTER INSERT ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts (rowid, content, normalized_content)
    VALUES (new.rowid, new.content, new.normalized_content);
END;

CREATE TRIGGER clipboard_items_ad AFTER DELETE ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts (clipboard_items_fts, rowid, content, normalized_content)
    VALUES ('delete', old.rowid, old.content, old.normalized_content);
END;

CREATE TRIGGER clipboard_items_au AFTER UPDATE ON clipboard_items BEGIN
    INSERT INTO clipboard_items_fts (clipboard_items_fts, rowid, content, normalized_content)
    VALUES ('delete', old.rowid, old.content, old.normalized_content);
    INSERT INTO clipboard_items_fts (rowid, content, normalized_content)
    VALUES (new.rowid, new.content, new.normalized_content);
END;

CREATE TABLE settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    launch_on_startup INTEGER NOT NULL DEFAULT 1 CHECK (launch_on_startup IN (0, 1)),
    enable_monitoring INTEGER NOT NULL DEFAULT 1 CHECK (enable_monitoring IN (0, 1)),
    max_history_size INTEGER NOT NULL DEFAULT 10000 CHECK (max_history_size >= 1)
);

INSERT INTO settings (id, launch_on_startup, enable_monitoring, max_history_size)
VALUES (1, 1, 1, 10000);
