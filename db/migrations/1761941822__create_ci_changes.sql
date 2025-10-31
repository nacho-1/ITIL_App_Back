CREATE TABLE ci_changes (
	id uuid NOT NULL DEFAULT gen_random_uuid(),
	ci_id uuid NOT NULL,
	PRIMARY KEY (id, ci_id),
	implementation_timedate TIMESTAMPTZ NOT NULL DEFAULT now(),
	documentation TEXT NOT NULL,
	CONSTRAINT fk_ci
		FOREIGN KEY (ci_id)
		REFERENCES configitems(id)
		ON DELETE CASCADE
);
