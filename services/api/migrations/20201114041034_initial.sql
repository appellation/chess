CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE public.users (
	id uuid NOT NULL DEFAULT gen_random_uuid(),
	CONSTRAINT users_pk PRIMARY KEY (id)
);

CREATE TABLE public.games (
	id uuid NOT NULL DEFAULT gen_random_uuid(),
	white_id uuid NOT NULL,
	black_id uuid NOT NULL,
	board varchar NOT NULL,
	moves _text NOT NULL DEFAULT ARRAY[]::character varying[],
	"result" varchar NULL,
	CONSTRAINT games_pk_1 PRIMARY KEY (id),
	CONSTRAINT games_black_fk FOREIGN KEY (black_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE SET NULL,
	CONSTRAINT games_white_fk FOREIGN KEY (white_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE SET NULL
);


CREATE TABLE public.user_accounts (
	user_id uuid NOT NULL,
	account_id varchar NOT NULL,
	account_type varchar NOT NULL,
	CONSTRAINT user_accounts_pk PRIMARY KEY (user_id, account_id, account_type),
	CONSTRAINT user_accounts_un UNIQUE (account_id, account_type),
	CONSTRAINT user_accounts_fk FOREIGN KEY (user_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE
);

CREATE OR REPLACE FUNCTION public.get_or_create_user(character varying, character varying)
 RETURNS users
 LANGUAGE plpgsql
AS $function$
	declare user_entry users%ROWTYPE;
	begin
		select * from users
			into user_entry
			right join user_accounts
			on users.id = user_accounts.user_id
			where user_accounts.account_id = $1
			and user_accounts.account_type = $2
			limit 1;
		if not found then
			insert into users values (default) returning * into strict user_entry;
			insert into user_accounts (user_id, account_id, account_type) values (user_entry.id, $1, $2);
		end if;
		return user_entry;
	END;
$function$
;
