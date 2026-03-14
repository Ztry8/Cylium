n = 1000000

a = 0
b = 1

for i = 0, n do
    local nxt = a + b
    a = b
    b = nxt
end