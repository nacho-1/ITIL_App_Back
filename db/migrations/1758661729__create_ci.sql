CREATE TYPE cistatus AS ENUM ('active', 'inactive', 'inmaintenance', 'testing', 'retired');

CREATE TABLE configitems (
	id uuid PRIMARY KEY default gen_random_uuid(),
	name varchar(255) NOT NULL,
	status cistatus NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
	type varchar(31),
	owner varchar(63),
	description varchar(255) NOT NULL
);

-- CREATE UNIQUE INDEX ci_id_idx ON configitems (id);
