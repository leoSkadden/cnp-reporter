// 28.059642, -80.590976

function main() {
	console.log('running main');
	let map = L.map('map').setView([28.059642, -80.590976], 13);

	L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
		maxZoom: 19,
		attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
	}).addTo(map);

	let popup = L.popup();
	let coords;

	function onMapClick(e) {
		const coordDisplayNode = document.getElementById("coord-display");
		coords = e.latlng;

		popup
			.setLatLng(coords)
			.setContent("Suspected CNP infection at " + coords.toString())
			.openOn(map);
		coordDisplayNode.innerText = stringFromCoords(coords);
	}

	map.on('click', onMapClick);

	function unshowStatusNode() {
		setTimeout(() => {
			const statusNode = document.getElementById("submit-status");
			statusNode.innerText = "Status";
			statusNode.style.display = "none";
		}, 10000);
	}

	async function onSubmitClick(e) {
		const responsePromise = fetch("https://backend.mangroves.report/api/v1/add-location", {
			method: "POST", // or 'PUT'
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify(coords),
		});

		const statusNode = document.getElementById("submit-status");

		const response = await responsePromise;

		if (response.ok) {
			// Update status node to show success
			statusNode.innerText = "Saved Successfully";
			statusNode.style.display = "block";
			unshowStatusNode();
		} else {
			// Update status node to show error
			statusNode.innerText = "Error could not save coordinate!";
			statusNode.style.display = "block";
		}
	}

	let submitButton = document.getElementById("submit-btn");
	submitButton.onclick = onSubmitClick;
}

function stringFromCoords(coords) {
	let lat = coords.lat.toFixed(6);
	let lng = coords.lng.toFixed(6);
	return `${lat}, ${lng}`
}

window.onload = main;