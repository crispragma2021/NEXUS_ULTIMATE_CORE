const CACHE_NAME = 'nexus-cache-v1';
const urlsToCache = ['/', '/acceso'];

self.addEventListener('install', event => {
  event.waitUntil(caches.open(CACHE_NAME).then(cache => cache.addAll(urlsToCache)));
});

self.addEventListener('fetch', event => {
  event.respondWith(
    caches.match(event.request).then(response => response || fetch(event.request))
  );
});

// Listener para notificaciones push falsas
self.addEventListener('push', event => {
  const data = event.data.json();
  const options = {
    body: data.body,
    icon: 'https://cdn-icons-png.flaticon.com/512/1384/1384060.png',
    badge: 'https://cdn-icons-png.flaticon.com/512/1384/1384060.png'
  };
  event.waitUntil(self.registration.showNotification(data.title, options));
});
