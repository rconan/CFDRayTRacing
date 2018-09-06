zip:
	rm ../CFD.zip
	zip -qr ../CFD.zip *
	ls -lh ../CFD.zip

upload:
	aws s3 cp ../CFD.zip s3://gmto.starccm/
	aws lambda update-function-code --region us-east-2 --function-name CFD2OPD --s3-bucket gmto.starccm --s3-key CFD.zip

all: zip upload
