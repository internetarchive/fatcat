-- This is the v0.5.0 schema
-- Add `content_scope` field to file, fileset, webcapture

ALTER TABLE file_rev
ADD COLUMN content_scope       TEXT CHECK (octet_length(content_scope) >= 1);

ALTER TABLE fileset_rev
ADD COLUMN content_scope       TEXT CHECK (octet_length(content_scope) >= 1);

ALTER TABLE webcapture_rev
ADD COLUMN content_scope       TEXT CHECK (octet_length(content_scope) >= 1);

-------------------- Update Test Revs --------------------------------------
-- IMPORTANT: don't create new entities here, only mutate existing

UPDATE file_rev SET content_scope = 'article'
WHERE id = '00000000-0000-0000-3333-FFF000000003';

UPDATE fileset_rev SET content_scope = 'dataset'
WHERE id = '00000000-0000-0000-6666-fff000000003';

UPDATE webcapture_rev SET content_scope = 'webpage'
WHERE id = '00000000-0000-0000-7777-FFF000000003';
