export const TWO_PI = Math.PI * 2
export const QUARTER_TURN = Math.PI / 2;


export function cartesianToLatLng([x, y, z]) {
	const radius = Math.sqrt(x ** 2 + y ** 2 + z ** 2);
	return [
		radius,
		(Math.PI / 2) - Math.acos(y / radius),
		normalizeAngle(Math.atan2(x, -z)),
	];
}

export function latLngToCartesian([radius, lat, lng]){
	lng = -lng + Math.PI / 2;
	return [
		radius * Math.cos(lat) * Math.cos(lng),
		radius * Math.sin(lat),
		radius * -Math.cos(lat) * Math.sin(lng),
	];
}


export function lerp(start, end, normalValue) {
	return start + (end - start) * normalValue;
}

export function inverseLerp(start, end, value) {
	return (value - start) / (end - start);
}