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
	let coordDisplayNode = document.getElementById("coord-display");

	function onMapClick(e) {
		coords = e.latlng;
		popup
			.setLatLng(coords)
			.setContent("Suspected CNP infection at " + coords.toString())
			.openOn(map);
		coordDisplayNode.innerText = stringFromCoords(coords);
	}

	map.on('click', onMapClick);
}

function stringFromCoords(coords) {
	console.log(coords.lat);
	console.log(coords.lng);
	let lat = coords.lat.toFixed(6);
	let lng = coords.lng.toFixed(6);
	return `${lat}, ${lng}`
}

window.onload = main;