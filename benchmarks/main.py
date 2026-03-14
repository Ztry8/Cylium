n = 1000000

a = 0
b = 1

for i in range(n + 1):
    nxt = a + b
    a = b
    b = nxt