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

	async function onSubmitClick(e) {
		const response = await fetch("http://170.187.157.232:8080/api/v1/add-location", {
			method: "POST", // or 'PUT'
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify(coords),
		});
	}

	let submitButton = document.getElementById("submit-btn");
	submitButton.onclick = onSubmitClick;
}

function stringFromCoords(coords) {
	console.log(coords.lat);
	console.log(coords.lng);
	let lat = coords.lat.toFixed(6);
	let lng = coords.lng.toFixed(6);
	return `${lat}, ${lng}`
}

window.onload = main;