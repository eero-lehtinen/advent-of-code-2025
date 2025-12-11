with open("input.txt") as f:
    lines = f.read().strip().split("\n")

dial = 50
zero_count = 0

for line in lines:
    if line[0] == "L":
        dir = -1
    else:
        dir = 1
    amount = int(line[1:])
    for _ in range(amount):
        dial = (dial + dir + 100) % 100
        if dial == 0:
            zero_count += 1

print("Answer:", zero_count)
