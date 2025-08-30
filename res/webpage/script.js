const songInput = document.getElementById('songs');
const songUpload = document.getElementById('song-submit');
const songListTable = document.getElementById('song-list-table');
const disableSongs = document.getElementById('disable-selected');
const enableSongs = document.getElementById('enable-selected');
const deleteSongs = document.getElementById('delete-selected');
const timetableForm = document.getElementById('timetable');
const addSongForm = document.getElementById('add-song-form');

// Songs to be disabled or deleted
const selectedSongs = [];

// URL Parameters
const params = new URLSearchParams(window.location.search);

const noRefresh = params.has('norefresh');

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
    selectedSongs.push(row.id.slice(5));
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

timetableForm.addEventListener("submit", e => {
    customSubmit(e, () => { if (!noRefresh) { location.reload(); } })
});
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

    // Collect form data
    const form = event.target;
    const formData = new FormData(form);

    // Example: send data with fetch instead of normal submit
    fetch(form.action, {
        method: form.method,
        body: formData
    })
        .then(response => response.text())
        .then(data => {
            console.log("Server response:", data);
        })
        .catch(error => {
            console.error("Error:", error);
        })
        .then(callback);
}

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

// Parse a css line to JS values
function csvToValue(csvLine) {
    if (csvLine.length === 0) {
        return [];
    }

    let result = [];

    for (const val of csvLine.split(',')) {
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

function arrayToCsv(arr) {
    let result = "";

    for (const val of arr) {
        if (val === null) {
            // Do nothing
        } else if (typeof (val) === 'string') {
            result += '"' + val + '"';
        } else if (typeof (val) === 'boolean') {
            result += String(val);
        } else if (typeof (val) === 'number') {
            result += String(val);
        }
        result += ',';
    }

    return result;
}

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
        const csvRows = csvText.trim().split("\r\n").map(line => csvToValue(line));
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
        const csvRows = csvText.trim().split("\r\n").map(line => csvToValue(line));

        for (let i = 0; i < 8; i++) {
            document.getElementById("break-start" + i).value = csvRows[i][0]
            document.getElementById("break-end" + i).value = csvRows[i][1]
        }
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
        const csvRows = csvText.trim().split("\r\n").map(line => csvToValue(line));

        for (const csvRow of csvRows) {
            if (csvRow.length === 0) {
                continue;
            }
            let row = songListTable.insertRow(-1);
            row.className = "song-list-row";
            row.id = "song-" + csvRow[0];
            row.insertCell(0).innerHTML = csvRow[0];
            if (csvRow[1]) {
                row.insertCell(1).innerHTML = "✔";
            } else {
                row.insertCell(1).innerHTML = "✘";
            }
        }
    })
    .catch(err => console.error("Fetch error:", err));
