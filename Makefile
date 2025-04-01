docker-image:
	docker build -f Dockerfile-server . -t up2pd

up2pd.tar: docker-image
	docker save up2pd > up2pd.tar

docker: up2pd.tar
	docker image rm up2pd

deploy:docker rm-image
	ssh_str=$$(cat ssh_str); \
	scp ./up2pd.tar "$$ssh_str:~/upload"; \
	ssh $$ssh_str "sudo docker load < ~/upload/up2pd.tar"

rm-image:
	ssh_str=$$(cat ssh_str); \
	ssh $$ssh_str "sudo docker image rm up2pd"

clean:
	rm up2pd.tar