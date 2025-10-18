CREATE TYPE problem_status AS ENUM ('open', 'knownerror', 'resolved', 'closed');

CREATE TABLE problems (
	id uuid PRIMARY KEY default gen_random_uuid(),
	title TEXT NOT NULL,
	status problem_status NOT NULL,
	detection_timedate TIMESTAMPTZ NOT NULL DEFAULT now(),
	description TEXT NOT NULL,
	causes TEXT NOT NULL,
	workarounds TEXT,
	resolutions TEXT
);

