const fs = require("fs");

const lines = fs.readFileSync("input.txt", "utf-8").trim().split("\n");

let dial = 50;
let zero_count = 0;

for (const line of lines) {
  const dir = line[0] === "L" ? -1 : 1;
  const amount = parseInt(line.slice(1), 10);
  for (let i = 0; i < amount; i++) {
    dial = (dial + dir + 100) % 100;
    if (dial === 0) {
      zero_count += 1;
    }
  }
}

console.log("Answer", zero_count);
