DROP TRIGGER IF EXISTS clipboard_items_ai;
DROP TRIGGER IF EXISTS clipboard_items_ad;
DROP TRIGGER IF EXISTS clipboard_items_au;
DROP TABLE IF EXISTS clipboard_items_fts;

CREATE VIRTUAL TABLE clipboard_items_fts USING fts5(
    content,
    normalized_content,
    content = 'clipboard_items',
    content_rowid = 'rowid',
    tokenize = 'unicode61',
    prefix = '2 3'
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

INSERT INTO clipboard_items_fts (clipboard_items_fts) VALUES ('rebuild');
