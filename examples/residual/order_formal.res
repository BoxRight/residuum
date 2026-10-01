module ResidualOrder

seeded verb a()
seeded verb b()
derived verb goal()

rule fewer(t) = a() @ t <= goal() @ t
rule more(t) = a() @ t & b() @ t <= goal() @ t
