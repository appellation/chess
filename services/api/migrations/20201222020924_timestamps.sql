ALTER TABLE public.games ADD created_at timestamp(0) NOT NULL DEFAULT now();
ALTER TABLE public.games ADD modified_at timestamp(0) NOT NULL DEFAULT now();

ALTER TABLE public.users ADD created_at timestamp(0) NOT NULL DEFAULT now();
ALTER TABLE public.users ADD modified_at timestamp(0) NOT NULL DEFAULT now();

ALTER TABLE public.user_accounts ADD created_at timestamp(0) NOT NULL DEFAULT now();
ALTER TABLE public.user_accounts ADD modified_at timestamp(0) NOT NULL DEFAULT now();
