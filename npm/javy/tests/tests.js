export async function simpleTest() {
	return "Yo";
}

export async function failingTest() {
	throw Error("HUH?");
}
