const MRC = [28.059642, -80.590976];

function main() {
	const map = L.map('map').setView(MRC, 13);
	const search = new GeoSearch.GeoSearchControl({
		provider: new GeoSearch.OpenStreetMapProvider(),
	});

	map.addControl(search);

	L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
		maxZoom: 19,
		attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
	}).addTo(map);

	const popup = L.popup();
	let coords;
	let data;

	map.on('click', (e) => {
		const coordDisplayNode = document.getElementById("coord-display");
		coords = e.latlng;

		popup
			.setLatLng(coords)
			.setContent("Suspected CNP infection at " + coords.toString())
			.openOn(map);
		coordDisplayNode.innerText = stringFromCoords(coords);
	});

	document.getElementById("submit-btn").onclick = async () => {
		data = { ...coords, images: [] };

		const imageInputNode = document.getElementById("image-input");
		let files = imageInputNode.files;

		if (files.length != 0) {
			for (const file of files) {
				const fileReader = new FileReader();

				const readPromise = new Promise((resolve, reject) => {
					fileReader.onloadend = () => resolve(fileReader.result);
					fileReader.onerror = reject;

					fileReader.readAsDataURL(file);
				})

				const result = await readPromise;

				data.images.push(result);
			}
		}

		const URL = "https://backend.mangroves.report";
		const responsePromise = fetch(`${URL}/api/v1/add-location`, {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
			},
			body: JSON.stringify(data),
		});

		const statusNode = document.getElementById("submit-status");

		const response = await responsePromise;

		if (response.ok) {
			// Update status node to show success
			statusNode.innerText = "Saved Successfully";
			statusNode.className = "visible";
			setTimeout(() => {
				const statusNode = document.getElementById("submit-status");
				statusNode.className = "hidden";
			}, 5000);
			setTimeout(() => {
				const statusNode = document.getElementById("submit-status");
				statusNode.innerText = "Status";
			}, 10000);
		} else {
			// Update status node to show error
			statusNode.innerText = "Error could not save coordinate!";
			statusNode.style.display = "block";
		}
	};

	navigator.geolocation.getCurrentPosition((geo) => {
		const location = [geo.coords.latitude, geo.coords.longitude]
		map.setView(location, 13);
	})
}

function stringFromCoords(coords) {
	return `${coords.lat.toFixed(6)}, ${coords.lng.toFixed(6)}`
}

window.onload = main;