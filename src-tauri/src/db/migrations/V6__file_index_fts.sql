-- FTS5 token index for file names (and extension) to scale search beyond naive LIKE.
CREATE VIRTUAL TABLE IF NOT EXISTS file_index_fts USING fts5(
  path UNINDEXED,
  name,
  extension,
  tokenize = 'porter unicode61'
);

CREATE TRIGGER IF NOT EXISTS file_index_ai AFTER INSERT ON file_index BEGIN
  INSERT INTO file_index_fts(rowid, path, name, extension)
  VALUES (new.rowid, new.path, new.name, new.extension);
END;

CREATE TRIGGER IF NOT EXISTS file_index_ad AFTER DELETE ON file_index BEGIN
  INSERT INTO file_index_fts(file_index_fts, rowid, path, name, extension)
  VALUES ('delete', old.rowid, old.path, old.name, old.extension);
END;

CREATE TRIGGER IF NOT EXISTS file_index_au AFTER UPDATE ON file_index BEGIN
  INSERT INTO file_index_fts(file_index_fts, rowid, path, name, extension)
  VALUES ('delete', old.rowid, old.path, old.name, old.extension);
  INSERT INTO file_index_fts(rowid, path, name, extension)
  VALUES (new.rowid, new.path, new.name, new.extension);
END;
