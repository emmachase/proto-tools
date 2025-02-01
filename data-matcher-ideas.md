
For each message:

For each field:
 - If it's the only field of a type, in both before and after, then it's 99% likely to be the same field.
 - If it's there are multiple fields of the same type, but the number of fields is unchanged, you can probably determine which one is which by comparing the values before and after.
   - This is assuming that both fields have unique values pre (and post)
   - If the values do not match pre and post (even though they are unique):
     - If the values have similar magnitudes, it is likely to be the same field (not 100% confidence though, mark for review):
       Eg:  uint32 A = 4
            uint32 B = 237842368432876

            uint32 A = 5
            uint32 B= 8234974892374

       Eg:  repeated uint32 A = [ /* 2 items */ ]
            repeated uint32 B = [ /* 7237 items */ ]

            repeated uint32 A = [ /* 3 items */ ]
            repeated uint32 B = [ /* 9234 items */ ]


To determine Struct identity:
 - Fallback: It seems that structs are usually placed in the same order in the protos before and after.

 - Initial method, if structs have the same number of fields pre-post, they are likely to be the same
   - Unless there are multiple unknown structs, each with the same number of fields
   - Can check type of sub-fields as well if they are resolved


==
Name Translation can be propagated to other messages, so once a field or type is resolved, 
make sure to update it in any other messages where it might be present.

==
Some Interesting Cases:

If MEMIECBAAJA is resolved, then we can transfer the name:
message OHJAIJKAADK {
	uint32 ONKCHDEMOCF = 4;
	oneof GHEIBKDHLPJ {
		MNMKMPMMOGN AMGPPOOFHLL = 6;
		HCAFGMCIGIH NHELBAHFOIH = 12;
		MAGHEDPCPOA FIHPGEEHMMB = 3;
		NNGOCPDILNC LOJCCIEIFPC = 2;
		GLGFGCLNIIL JLCFPKEHLJD = 13;
		DDKJLLPJNOB DCFNCDEIEOM = 10;
		MEMIECBAAJA PUNK_LORD_SHARE_TYPE_FRIEND = 1;
		DJMOGJKNMPK FLOPGKPHEOP = 15;
		CKMHLMLLEHD HKOFFLPCLOG = 7;
	}
}

This one seems like it would be really hard:
message GJODAPFIFML {
	oneof ECMJGOKIKOM {
		bool MEFFKCAPBFJ = 13;
		bool KOFDEOAGLGK = 12;
		bool DENFLJKHNFO = 8;
		bool KHKAAPACGJF = 7;
		bool KKHHCOPENGM = 6;
		bool KMCCKIMHBBJ = 11;
		bool OIGIPMEOEKB = 1;
		bool KMACDMAFFFN = 15;
		bool EFCCDDMNMLP = 2;
		bool EBEPHGLDCNH = 10;
	}
}

