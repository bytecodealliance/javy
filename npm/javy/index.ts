interface JavyBuiltins {
	IO: {
		// readSync: Similar to `write` in POSIX
		//
		// Params:
		// - fd: File Descriptor (0 = stdin, 1 = stdout, 2 = stderr, >2 = custom)
		// - buffer: Buffer to read into
		//
		// Return:
		//   - > 0: Number of bytes read
		//   - = 0: EOF reached
		//   - < 0: Error occured
		readSync(fd: number, buffer: Uint8Array): number;
		// writeSync: Similar to `write` in POSIX
		//
		// Params:
		// - fd: File Descriptor (0 = stdin, 1 = stdout, 2 = stderr, >2 = custom)
		// - buffer: Buffer to write
		//
		// Return:
		//   - >= 0: Number of bytes written
		//   - < 0: Error occured
		writeSync(fd: number, buffer: Uint8Array): number;
	};
}

declare global {
	const Javy: JavyBuiltins;
}

export const enum STDIO {
	Stdin,
	Stdout,
	Stderr,
}
