CREATE TYPE incident_status AS ENUM ('open', 'inprogress', 'closed');
CREATE TYPE incident_impact AS ENUM ('high', 'medium', 'low');
CREATE TYPE incident_urgency AS ENUM ('high', 'medium', 'low');

CREATE TABLE incidents (
	id uuid PRIMARY KEY default gen_random_uuid(),
	title TEXT NOT NULL,
	status incident_status NOT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
	resolved_at TIMESTAMPTZ,
	impact incident_impact NOT NULL,
	urgency incident_urgency NOT NULL,
	owner TEXT,
	description TEXT NOT NULL
);

