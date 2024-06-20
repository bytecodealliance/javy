const input = JSON.parse('{"elements":[{"id":"zza"},{"id":"zzb"},{"id":"zzc"},{"id":"zzd"},{"id":"zze"}]}');

const acc = {};
input.elements.forEach(e => {
  if (!acc[e.id]) {
    acc[e.id] = 1;
  }
});
