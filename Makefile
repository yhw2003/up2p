docker-image:
	docker build -f Dockerfile-server . -t up2pd

docker-save: docker-image
	docker save up2pd > up2pd.tar

docker: docker-save
	docker image rm up2pd

clean:
	rm up2pd.tar