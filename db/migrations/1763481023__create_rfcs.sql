CREATE TYPE rfcstatus AS ENUM ('open', 'inprogress', 'closed');

CREATE TABLE rfcs (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	title TEXT NOT NULL,
	status rfcstatus NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
	finished_at TIMESTAMPTZ,
	requester TEXT NOT NULL,
	description TEXT NOT NULL
);

