const songInput = document.getElementById('songs');
const songUpload = document.getElementById('song-submit');
const songListTable = document.getElementById('song-list-table');
const disableSongs = document.getElementById('disable-selected');
const enableSongs = document.getElementById('enable-selected');
const deleteSongs = document.getElementById('delete-selected');
const playSongs = document.getElementById('play-selected');
const timetableForm = document.getElementById('timetable');
const addSongForm = document.getElementById('add-song-form');
const utcOffset = document.getElementById("utc-offset");
const timeDisplay = document.getElementById("server-time");

// Songs to be disabled or deleted
const selectedSongs = [];

// URL Parameters
const params = new URLSearchParams(window.location.search);

const noRefresh = params.has('norefresh');

const defaultSeparator = ',';
const defaultStrMarker = '"';

// Fetches a url as and returns its text response
async function fetchText(url) {
    // Bounds checking omitted, I know what I'm doing

    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`Fetch failed with status ${response.status}: ${response.statusText}`);
    }

    return await response.text();
}

// Song upload
songInput.addEventListener('change', function () {
    const count = songInput.files.length;
    if (count === 0) {
        songUpload.disabled = true;
        songUpload.value = 'Upload 0 song';
    } else if (count === 1) {
        songUpload.disabled = false;
        songUpload.value = 'Upload 1 song';
    } else {
        songUpload.disabled = false;
        songUpload.value = 'Upload ' + count + ' songs';
    }
});

// Select song from table
songListTable.addEventListener('click', function (e) {
    let row = e.target.closest('tr');
    row.classList.toggle('active');

    let filename = row.id.slice(5);

    let arrayIndex =selectedSongs.indexOf(filename);

    if (arrayIndex === -1) {
        selectedSongs.push(filename);
    } else {
        selectedSongs.splice(arrayIndex, 1);
    }
});

// Disable selected songs
disableSongs.addEventListener('click', function (e) {
    if (selectedSongs.length === 0) {
        return;
    }

    const response = fetch(
        "/api/disable-songs", {
        method: 'POST',
        headers: {
            "Content-Type": "application/csv"
        },
        body: arrayToCsv(selectedSongs)
    });

    if (!(response.ok || (response.status === undefined))) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }

    if (!noRefresh) { location.reload(); }
});

// Enable selected songs
enableSongs.addEventListener('click', function (e) {
    if (selectedSongs.length === 0) {
        return;
    }

    const response = fetch(
        "/api/enable-songs", {
            method: 'POST',
            headers: {
                "Content-Type": "application/csv"
            },
            body: arrayToCsv(selectedSongs)
        });

    if (!(response.ok || (response.status === undefined))) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }

    if (!noRefresh) { location.reload(); }
});

// Delete selected songs
deleteSongs.addEventListener('click', function (e) {
    if (selectedSongs.length === 0) {
        return;
    }

    const response = fetch(
        "/api/delete-songs", {
            method: 'POST',
            headers: {
                "Content-Type": "application/json"
            },
            body: arrayToCsv(selectedSongs)
        });

    if (!(response.ok || (response.status === undefined))) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }

    if (!noRefresh) { location.reload(); }
});

// Delete selected songs
playSongs.addEventListener('click', function (e) {
    if (selectedSongs.length === 0) {
        return;
    }

    const response = fetch(
        "/api/play-songs", {
            method: 'POST',
            headers: {
                "Content-Type": "application/json"
            },
            body: arrayToCsv(selectedSongs)
        });

    if (!(response.ok || (response.status === undefined))) {
        throw new Error(`HTTP error! Status: ${response.status}`);
    }

    if (!noRefresh) { location.reload(); }
});

// Submit timetable
timetableForm.addEventListener("submit", e => {
    customSubmit(e, () => { if (!noRefresh) { location.reload(); } })
});

// Submit songs
addSongForm.addEventListener("submit", e => {
    songUpload.disabled = true;
    songUpload.value = "Uploading...";
    submitMultipleFiles(e, () => {
        e.target.reset();
        songUpload.value = 'Upload 0 song';
        if (!noRefresh) { location.reload(); }
    });
});

// Submits a form
function customSubmit(event, callback = () => {}) {
    // Prevent default form submission
    event.preventDefault();

    const checkboxes = document.querySelectorAll("#timetable input[type=checkbox]");
    const times = document.querySelectorAll("#timetable input[type=time]");
    const offset = document.getElementById("utc-offset");

    const checkboxLines = [];
    for (let row = 0; row < 8; row++) {
        const rowArr = [];
        for (let col = 0; col < 5; col++) {
            const index = row * 5 + col; // row-major index
            rowArr.push(checkboxes[index].checked);
        }
        checkboxLines.push(arrayToCsv(rowArr));
    }

    const timeLines = [];
    for (let row = 0; row < 8; row++) {
        const rowArr = [];
        for (let col = 0; col < 2; col++) {
            const index = row * 2 + col; // row-major index
            rowArr.push(times[index].value);
        }
        timeLines.push(arrayToCsv(rowArr));
    }

    // Collect form data
    const form = event.target;

    let finished = 0;

    fetch("/api/set-timetable", {
        method: form.method,
        body: checkboxLines.join('\r\n')
    })
        .then(response => response.text())
        .then(data => {
            console.log("Server response:", data);
        })
        .catch(error => {
            console.error("Error:", error);
        })
        .then(() => {
            finished++;
            if (finished === 3) {
                callback();
            }
        });

    fetch("/api/set-breaks", {
        method: form.method,
        body: timeLines.join('\r\n')
    })
        .then(response => response.text())
        .then(data => {
            console.log("Server response:", data);
        })
        .catch(error => {
            console.error("Error:", error);
        })
        .then(() => {
            finished++;
            if (finished === 3) {
                callback();
            }
        });

    fetch("/api/set-utc-offset", {
        method: form.method,
        body: String(offset.value)
    })
        .then(response => response.text())
        .then(data => {
            console.log("Server response:", data);
        })
        .catch(error => {
            console.error("Error:", error);
        })
        .then(() => {
            finished++;
            if (finished === 3) {
                callback();
            }
        });
}

// Submits multiple files
function submitMultipleFiles(event, callback = () => {}) {
    event.preventDefault();

    const files = event.target.songs.files;
    let finished = 0;

    for (const file of files) {
        const prefix = new TextEncoder().encode(file.name + ':');
        const payload = new Blob([prefix, file], { type: "application/octet-stream" });

        fetch("/api/add-song", {
            method: 'POST',
            headers: {
                "Content-Type": "application/octet-stream"
            },
            body: payload
        }).then(() => {
            finished++
            if (finished === files.length) {
                callback();
            }
        });
    }
}

// Split a string at the given separator values, while skipping splitting inside the markers
function splitWithMarker(input, sep, str_mkr) {
    const result = [];
    let current = "";

    let in_str = false;

    for (let i = 0; i < input.length; i++) {
        if (input[i] === str_mkr) { in_str = !in_str; }
        if (!in_str && (input[i] === sep)) {
            result.push(current);
            current = "";
        } else {
            current += input[i];
        }
    }

    result.push(current);

    return result;
}

// Parses a CSS line to JS values
function csvToValue(csvLine, sep, str_mkr) {
    if (csvLine.length === 0) {
        return [];
    }

    let result = [];

    let in_str = false;
    const split = splitWithMarker(csvLine, sep, str_mkr);

    for (const val of split) {
        if (val === '') {
            result.push(null);
        } else if ((val[0] === '\"') && (val[val.length - 1] === '\"')) {
            result.push(val.slice(1, -1));
        } else if (val === 'true') {
            result.push(true);
        } else if (val === 'false') {
            result.push(false);
        } else {
            result.push(Number(val));
        }
    }

    return result;
}

// Parses an array of JS values into CSV line
function arrayToCsv(arr) {
    let result = [];

    for (const val of arr) {
        if (val === null) {
            // Do nothing
        } else if (typeof (val) === 'string') {
            result.push('"' + val + '"');
        } else if (typeof (val) === 'boolean') {
            result.push(String(val));
        } else if (typeof (val) === 'number') {
            result.push(String(val));
        }
    }

    return result.join(',');
}

async function main() {
    // Query server time
    let serverTime = Number(await fetchText("/data/server-time-seconds"));

    setInterval(() => {
        serverTime++;
        const h = String(Math.floor(serverTime / 3600)).padStart(2, '0');
        const m = String(Math.floor((serverTime % 3600) / 60)).padStart(2, '0');
        const s = String(serverTime % 60).padStart(2, '0');
        timeDisplay.innerHTML = `${h}:${m}:${s}`
    }, 1000);

    // Fetch the timetable
    fetch("data/timetable.csv")
        .then(res => {
            if (!res.ok) {
                throw new Error("HTTP Error" + res.status);
            }
            return res.text();
        })
        .then(csvText => {
            // Simple CSV parsing (splitting by newlines and commas)
            const csvRows = csvText.trim().split("\r\n").map(line => csvToValue(line, defaultSeparator, defaultStrMarker));
            const days = ["mon", "tue", "wed", "thu", "fri"];

            for (let i = 0; i < 5; i++) {
                for (let j = 0; j < 8; j++) {
                    document.getElementById(days[i] + j).checked = csvRows[j][i];
                }
            }
        })
        .catch(err => console.error("Fetch error:", err));

    // Fetch the breaks
    fetch("data/breaks.csv")
        .then(res => {
            if (!res.ok) {
                throw new Error("HTTP Error" + res.status);
            }
            return res.text();
        })
        .then(csvText => {
            // Simple CSV parsing (splitting by newlines and commas)
            const csvRows = csvText.trim().split("\r\n").map(line => csvToValue(line, defaultSeparator, defaultStrMarker));

            for (let i = 0; i < 8; i++) {
                document.getElementById("break-start" + i).value = csvRows[i][0]
                document.getElementById("break-end" + i).value = csvRows[i][1]
            }
        })
        .catch(err => console.error("Fetch error:", err));

    // Fetch UTC offset
    fetch("data/utc-offset.bin")
        .then(res => {
            if (!res.ok) {
                throw new Error("HTTP Error" + res.status);
            }
            return res.arrayBuffer();
        })
        .then(buf => {
            const int8View = new Int8Array(buf);

            const num = int8View[0];

            if (buf.byteLength !== 1 || num < -24 || num > 24) {
                throw new Error("Invalid response")
            }

            utcOffset.value = num
        })
        .catch(err => console.error("Fetch error:", err));

    // Fetch the song list
    fetch("data/songs.csv")
        .then(res => {
            if (!res.ok) {
                throw new Error("HTTP Error" + res.status);
            }
            return res.text();
        })
        .then(csvText => {
            const csvRows = csvText.trim().split("\r\n").map(line => csvToValue(line, defaultSeparator, defaultStrMarker));

            for (const csvRow of csvRows) {
                if (csvRow.length === 0) {
                    continue;
                }
                let row = songListTable.insertRow(-1);
                row.className = "song-list-row";
                row.id = "song-" + csvRow[0];

                const filenameCell = row.insertCell(0);
                filenameCell.innerHTML = csvRow[0];
                filenameCell.className = "filename-field";

                if (csvRow[1]) {
                    row.insertCell(1).innerHTML = "✔";
                } else {
                    row.insertCell(1).innerHTML = "✘";
                }
            }
        })
        .catch(err => console.error("Fetch error:", err));
}

main();