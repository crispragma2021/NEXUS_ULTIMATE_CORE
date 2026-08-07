
function luhnCheck(num) {
    let arr = (num + '').split('').reverse().map(x => parseInt(x));
    let lastDigit = arr.splice(0, 1)[0];
    let sum = arr.reduce((acc, val, i) => (i % 2 !== 0 ? acc + val : acc + ((val * 2) % 9 || (val * 2 === 9 ? 9 : 0))), 0);
    sum += lastDigit;
    return sum % 10 === 0;
}

function generateLuhn(bin) {
    let num = bin;
    while (num.length < 15) {
        num += Math.floor(Math.random() * 10);
    }
    let arr = num.split('').reverse().map(x => parseInt(x));
    let sum = arr.reduce((acc, val, i) => (i % 2 === 0 ? acc + ((val * 2) % 9 || (val * 2 === 9 ? 9 : 0)) : acc + val), 0);
    let checkDigit = (10 - (sum % 10)) % 10;
    return num + checkDigit;
}

const bin = "489504";
console.log("--- NEXUS BIN GENERATOR (489504) ---");
for(let i=0; i<5; i++) {
    const cc = generateLuhn(bin);
    const expM = Math.floor(Math.random() * 12) + 1;
    const expY = 2027 + Math.floor(Math.random() * 3);
    const cvv = Math.floor(Math.random() * 899) + 100;
    console.log(`CC: ${cc} | EXP: ${expM.toString().padStart(2, '0')}/${expY} | CVV: ${cvv}`);
}
