module ResidualAlternatives

seeded verb a()
seeded verb b()
seeded verb c()
seeded verb d()
derived verb left()
derived verb right()
derived verb goal()

rule fromA(t) = a() @ t <= left() @ t
rule fromB(t) = b() @ t <= left() @ t
rule fromC(t) = c() @ t <= right() @ t
rule fromD(t) = d() @ t <= right() @ t
rule combine(t) = left() @ t & right() @ t <= goal() @ t
