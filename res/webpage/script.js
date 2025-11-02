const songInput = document.getElementById('songs');
const songUpload = document.getElementById('song-submit');

const songListTable = document.getElementById('song-list-table');

const disableSongs = document.getElementById('disable-selected');
const enableSongs = document.getElementById('enable-selected');
const deleteSongs = document.getElementById('delete-selected');
const playSongs = document.getElementById('play-selected');

const timetableForm = document.getElementById('timetable');

const addSongForm = document.getElementById('add-song-form');

const utcOffset = document.getElementById("utc-offset-number");

const timeDisplay = document.getElementById("server-time");

const eventListTable = document.getElementById('event-list-table');

const addEventForm = document.getElementById('add-event-form');
const eventName = document.getElementById("event-name");
const eventSound = document.getElementById("event-sound");
const scheduledSwitch = document.getElementById("scheduled-switch");
const eventTriggerTime = document.getElementById("event-trigger-time");
const eventRepeatTime = document.getElementById("event-repeat-time");
const eventRepeatAmount = document.getElementById("event-repeat-amount");
const eventAutodeleteSwitch = document.getElementById("event-autodelete-switch");
const addEvent = document.getElementById("add-event");

// Songs to be disabled or deleted
const selectedSongs = [];

// The event selected to be edited
let selectedEvent = null; // Disgusting `null` -> `Option<String>`

// Represents the state of the switch that select the event scheduling
let addEventSchedulingSet = false;
let eventAutoDeleteSet = false;
let eventSchedulingElems = [eventTriggerTime, eventRepeatTime, eventRepeatAmount];

// URL Parameters
const params = new URLSearchParams(window.location.search);

const noRefresh = params.has('norefresh');

const defaultSeparator = ',';
const defaultStrMarker = '"';

const eventCreationOptions = addEventForm.innerHTML;

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

    let arrayIndex = selectedSongs.indexOf(filename);

    if (arrayIndex === -1) {
        selectedSongs.push(filename);
    } else {
        selectedSongs.splice(arrayIndex, 1);
    }
});

// Select event from table
eventListTable.addEventListener('click', function (e) {
    let row = e.target.closest('tr');
    row.classList.toggle('active');

    let name = row.id.slice(6);

    if (selectedEvent === name) {
        selectedEvent = null

        addEventForm.innerHTML = eventCreationOptions;
    } else {
        selectedEvent = name

        addEventForm.innerHTML =
            "<input type=\"submit\" class=\"submit dangerous\" value=\"Remove Event\" id=\"remove-event\">";
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
                "Content-Type": "application/csv"
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
    timetableSubmit(e, () => { if (!noRefresh) { location.reload(); } })
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

addEventForm.addEventListener("submit", e => {
    if (selectedEvent === null) {
        addEventSubmit(e, () => { if (!noRefresh) { location.reload(); } })
    } else {
        removeEventSubmit(e, () => { if (!noRefresh) { location.reload(); } })
    }
})

// Make the schedule switch togglable
scheduledSwitch.addEventListener('click', function (e) {
    // Toggle switch state
    if (addEventSchedulingSet) {
        scheduledSwitch.classList.remove('switched-on');
    } else {
        scheduledSwitch.classList.add('switched-on');
    }

    // Toggle scheduling elements
    for (const elem of eventSchedulingElems) {
        elem.disabled = addEventSchedulingSet;
    }

    // Toggle global state
    addEventSchedulingSet = !addEventSchedulingSet;

    addEvent.disabled = !validateNewEvent()
})

eventAutodeleteSwitch.addEventListener('click', function (e) {
    if (addEventSchedulingSet) {
        // Toggle switch state
        if (eventAutoDeleteSet) {
            eventAutodeleteSwitch.classList.remove('switched-on');
        } else {
            eventAutodeleteSwitch.classList.add('switched-on');
        }

        // Toggle global state
        eventAutoDeleteSet = !eventAutoDeleteSet;
    }

    addEvent.disabled = !validateNewEvent()
})

eventName.addEventListener('change', function (e) {
    addEvent.disabled = !validateNewEvent()
})

eventSound.addEventListener('change', function (e) {
    addEvent.disabled = !validateNewEvent()
})

eventTriggerTime.addEventListener('change', function (e) {
    addEvent.disabled = !validateNewEvent()
})

eventRepeatTime.addEventListener('change', function (e) {
    addEvent.disabled = !validateNewEvent()
})

eventRepeatAmount.addEventListener('change', function (e) {
    addEvent.disabled = !validateNewEvent()
})

function validateNewEvent() {
    const namelen = eventName.value.length;
    const nEventRepeatTime = Number(eventRepeatTime.value)
    const nEventRepeatAmount = Number(eventRepeatAmount.value)

    return (
        (namelen > 0 && namelen <= 16) && // Name is valid
        eventSound.files.length === 1 && // There is a file selected
        (!addEventSchedulingSet || ( // If the event has scheduling...
            eventTriggerTime.value !== "" && // A trigger time is selected
            (nEventRepeatTime >= 0 && nEventRepeatTime <= 18446744073709551615) && // Event repeat time is in-bounds
            (nEventRepeatAmount >= 0 && nEventRepeatAmount <= 65535) // Event repeat amount is in-bounds
        ))
    );
}

// Submits the timetable form
function timetableSubmit(event, callback = () => {}) {
    // Prevent default form submission
    event.preventDefault();

    const checkboxes = document.querySelectorAll("#timetable input[type=checkbox]");
    const times = document.querySelectorAll("#timetable input[type=time]");
    const offset = document.getElementById("utc-offset-number");

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

// Submits a new event
function addEventSubmit(event, callback = () => {}) {
    event.preventDefault();

    // ## Serialize string data ##
    let serializedUtf8 = eventName.value;
    if (addEventSchedulingSet) {
        serializedUtf8 += "\0";
        serializedUtf8 += eventTriggerTime.value;
        serializedUtf8 += "\0";
        serializedUtf8 += eventRepeatTime.value;
        serializedUtf8 += "\0";
        serializedUtf8 += eventRepeatAmount.value;
        if (eventAutoDeleteSet) {
            serializedUtf8 += "T";
        } else {
            serializedUtf8 += "F";
        }
    }
    serializedUtf8 += "\n";

    // ## Creating payload ##
    const prefix = new TextEncoder().encode(serializedUtf8);
    const payload = new Blob([prefix, eventSound.files[0]], { type: "application/octet-stream" });

    // ## Submitting ##
    fetch("/api/add-event", {
        method: 'POST',
        headers: {
            "Content-Type": "application/octet-stream"
        },
        body: payload
    }).then(callback);
}

// Removes an event
function removeEventSubmit(event, callback = () => {}) {
    event.preventDefault();

    const csv = arrayToCsv([selectedEvent]);

    // ## Submitting ##
    fetch("/api/remove-events", {
        method: 'POST',
        headers: {
            "Content-Type": "application/csv"
        },
        body: csv
    }).then(callback);
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

                const enabledCell = row.insertCell(1);
                if (csvRow[1]) {
                    enabledCell.innerHTML = "✔";
                } else {
                    enabledCell.innerHTML = "✘";
                }
                enabledCell.className = "enabled-field";
            }
        })
        .catch(err => console.error("Fetch error:", err));

    // Fetch the event list
    fetch("data/events.csv")
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
                let row = eventListTable.insertRow(-1);
                row.className = "event-list-row";
                row.id = "event-" + csvRow[0];

                const nameCell = row.insertCell(0);
                nameCell.innerHTML = csvRow[0];
                nameCell.className = "name-field";

                const timeCell = row.insertCell(1);
                timeCell.innerHTML = csvRow[1];
                timeCell.className = "time-field";
            }
        })
        .catch(err => console.error("Fetch error:", err));
}

main();