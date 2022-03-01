const fs = require("fs");
const { argv } = require("process");

const file_path = argv[2];
const type = argv[3];

const lines = fs.readFileSync(file_path, 'utf-8').split('\n');
let result = ['\n'];

// (?<!struct ) = no strarting with `struct `
const findStartRegex = `(?<!struct )${type} {`;

const process = () => {
    for (let current_line_index = 0; current_line_index < lines.length; current_line_index++) {
        const current_line = lines[current_line_index];

        const match = current_line.match(new RegExp(findStartRegex));
        const foundStart = match?.length > 0;

        if (foundStart) {
            const { replacements, next_index } = replace(current_line_index, match)
            current_line_index = next_index;

            result = result.concat(replacements);
            continue
        }

        result.push(current_line);
    }

    return result;
}

const replace = (start_index, startMatch) => {
    let changedResult = [];
    let notChangedResult = [];
    const findEndRegex = `^${new Array(startMatch.index + 1).join(' ')}}`;
    // ^:  = ends with ':' or ' '
    let indent = new Array(startMatch.index + 1).join(' ');
    const findFalseEndRegex = `^${indent}}[: ]`;

    let current_line_index = start_index;
    let firstLine = lines[start_index];

    const startOfFieldRegex = `${indent}[\\w_]+:`;
    const endOfFieldRegex = `,$`;

    changedResult.push(firstLine.replace(new RegExp(findStartRegex), `inline_init(|v: &mut ${type}|{`))
    notChangedResult.push(firstLine)

    for (let current_line_index = start_index + 1; current_line_index < lines.length; current_line_index++) {
        const current_line = lines[current_line_index];

        let matchEndRegex = current_line.match(new RegExp(findEndRegex));
        const foundEnd = matchEndRegex?.length > 0;
        let matchFalseEndRegex = current_line.match(new RegExp(findFalseEndRegex));
        const foundFalseEnd = matchFalseEndRegex?.length > 0;

        if (foundFalseEnd) {
            return { replacements: notChangedResult, next_index: current_line_index };
        }

        if (foundEnd) {
            changedResult.push(current_line.replace(new RegExp(findEndRegex), '})'));
            break;
        }

        notChangedResult.push(current_line);

        let startOfFieldOriginalMatch = current_line.match(new RegExp(startOfFieldRegex));
        let foundStartOfField = startOfFieldOriginalMatch?.length > 0;

        let changed_line = current_line;
        if (foundStartOfField) {
            let startOfFieldOriginal = startOfFieldOriginalMatch[0]

            let startOfField = startOfFieldOriginal.slice(0, -1)
            console.log(startOfField)

            changed_line = current_line.replace(startOfFieldOriginal, `v.${startOfField} = `);
        }

        let endOfFiedMatch = changed_line.match(new RegExp(endOfFieldRegex));
        let foundEndOfField = endOfFiedMatch?.length > 0;

        if (foundEndOfField) {
            changed_line = `${changed_line.slice(0, -1)};`
        }
        changedResult.push(changed_line);

    }

    return { replacements: changedResult, next_index: current_line_index };
}



// console.log('                    id: stocktake_line_a.id.clone(),'.match(new RegExp(' [\\w_]+:')))

console.log(process().join('\n'));