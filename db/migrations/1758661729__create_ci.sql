CREATE TYPE cistatus AS ENUM ('active', 'inactive', 'maintenance', 'testing', 'retired');

CREATE TABLE configitems (
	id uuid PRIMARY KEY default gen_random_uuid(),
	name TEXT NOT NULL,
	status cistatus NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
	type TEXT,
	owner TEXT,
	description TEXT NOT NULL
);

-- CREATE UNIQUE INDEX ci_id_idx ON configitems (id);
